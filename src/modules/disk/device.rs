use lsblk::BlockDevice;

const BY_ID: &str = "/dev/disk/by-id";

#[derive(Debug, Clone)]
pub struct Disk {
    pub name: String,
    pub id: Option<String>,
    pub size: u64,
    pub partitions: Vec<Partition>,
}

#[derive(Debug, Clone)]
pub struct Partition {
    pub name: String,
    pub id: Option<String>,
    pub size: u64,
}

impl Disk {
    /// Enumerate all physical disks that have a stable by-id entry, grouped
    /// with their partitions. Requires no elevated privileges.
    pub fn list() -> Vec<Disk> {
        let all = BlockDevice::list().unwrap_or_default();

        let mut disks: Vec<Disk> = all
            .iter()
            .filter(|d| d.is_disk() && d.id.is_some())
            .map(|d| Disk {
                name: d.name.clone(),
                id: d.id.clone(),
                size: capacity_bytes(d),
                partitions: vec![],
            })
            .collect();

        disks.sort_by(|a, b| a.name.cmp(&b.name));

        for disk in &mut disks {
            let mut parts: Vec<Partition> = all
                .iter()
                .filter(|d| d.is_part())
                .filter(|d| d.disk_name().is_ok_and(|parent| parent == disk.name))
                .map(|d| Partition {
                    name: d.name.clone(),
                    id: d.id.clone(),
                    size: capacity_bytes(d),
                })
                .collect();
            parts.sort_by(|a, b| a.name.cmp(&b.name));
            disk.partitions = parts;
        }

        disks
    }

    /// Full by-id path, e.g. `/dev/disk/by-id/ata-Samsung_860_EVO_xxx`.
    /// Falls back to `/dev/{name}` if no by-id entry exists.
    pub fn id_path(&self) -> String {
        self.id
            .as_deref()
            .map(|id| format!("{BY_ID}/{id}"))
            .unwrap_or_else(|| format!("/dev/{}", self.name))
    }
}

impl Partition {
    /// Full by-id path, e.g. `/dev/disk/by-id/ata-Samsung_860_EVO_xxx-part1`.
    /// Falls back to `/dev/{name}`.
    pub fn id_path(&self) -> String {
        self.id
            .as_deref()
            .map(|id| format!("{BY_ID}/{id}"))
            .unwrap_or_else(|| format!("/dev/{}", self.name))
    }
}

fn capacity_bytes(bd: &BlockDevice) -> u64 {
    bd.capacity()
        .ok()
        .flatten()
        .map(|sectors| sectors * 512)
        .unwrap_or(0)
}
