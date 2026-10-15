mod block;
mod drive;
mod manager;
mod partition_table;

use std::collections::HashMap;

use block::BlockProxy;
use manager::ManagerProxy;
use partition_table::PartitionTableProxy;
use zbus::{
    Connection,
    zvariant::{OwnedObjectPath, Signature},
};

#[derive(Debug)]
pub struct Disk {
    pub size: u64,
    pub name: String,
    pub partitions: Vec<DiskPartition>,
}

#[derive(Debug)]
pub struct DiskPartition {
    pub size: u64,
    pub name: String,
    pub format: String,
}

pub async fn get_disks() -> anyhow::Result<Vec<Disk>> {
    let conn = Connection::system().await?;
    let manager = ManagerProxy::new(&conn).await?;

    let mut drives = vec![];

    let block_paths = manager.get_block_devices(HashMap::new()).await?;

    for block_path in block_paths {
        let block_proxy = get_block_proxy(&conn, block_path.clone()).await?;

        let introspection = block_proxy.inner().introspect().await?;

        if !introspection.contains("<interface name=\"org.freedesktop.UDisks2.PartitionTable\">") {
            continue;
        }

        let mut partitions: Vec<DiskPartition> = vec![];

        let partition_table = PartitionTableProxy::builder(&conn)
            .path(block_path)?
            .build()
            .await?;

        for partition_path in partition_table.partitions().await? {
            let block_proxy = get_block_proxy(&conn, partition_path).await?;

            let configuration = block_proxy.configuration().await?;
            let configuration = configuration
                .into_iter()
                .find(|configuration| configuration.0 == "fstab")
                .map(|configuration| configuration.1);

            let drive_partition = DiskPartition {
                size: block_proxy.size().await?,
                name: String::from_utf8(block_proxy.device().await?)?.replace("\0", ""),
                format: configuration
                    .and_then(|conf| conf.get("type").cloned())
                    .and_then(|type_| match type_.into() {
                        zbus::zvariant::Value::Array(array) => {
                            if *array.element_signature() != Signature::U8 {
                                return None;
                            }

                            String::from_utf8(
                                array
                                    .iter()
                                    .filter_map(|value| match value {
                                        zbus::zvariant::Value::U8(value) => Some(*value),
                                        _ => None,
                                    })
                                    .collect(),
                            )
                            .ok()
                        }
                        _ => None,
                    })
                    .unwrap_or_else(|| "unknown".to_string())
                    .replace("\0", ""),
            };

            partitions.push(drive_partition);
        }

        let drive = Disk {
            partitions,
            size: block_proxy.size().await?,
            name: String::from_utf8(block_proxy.device().await?)?.replace("\0", ""),
        };

        drives.push(drive);
    }

    Ok(drives)
}

async fn get_block_proxy<'a>(
    conn: &'a Connection,
    path: OwnedObjectPath,
) -> zbus::Result<BlockProxy<'a>> {
    BlockProxy::builder(conn).path(path)?.build().await
}
