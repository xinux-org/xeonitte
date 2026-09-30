use std::marker::PhantomData;

// TODO: need add some ergonomics and Unit constructors for consntructing Size<UnitType>
pub trait SizeUnit: 'static {
    const BYTES: u64;
    const DISKO_SUFFIX: &'static str;
}

#[derive(Debug, Clone, Copy)]
pub struct KiB;
#[derive(Debug, Clone, Copy)]
pub struct MiB;
#[derive(Debug, Clone, Copy)]
pub struct GiB;
#[derive(Debug, Clone, Copy)]
pub struct TiB;

impl SizeUnit for KiB {
    const BYTES: u64 = 1_024;
    const DISKO_SUFFIX: &'static str = "K";
}
impl SizeUnit for MiB {
    const BYTES: u64 = 1_024 * 1_024;
    const DISKO_SUFFIX: &'static str = "M";
}
impl SizeUnit for GiB {
    const BYTES: u64 = 1_024 * 1_024 * 1_024;
    const DISKO_SUFFIX: &'static str = "G";
}
impl SizeUnit for TiB {
    const BYTES: u64 = 1_024 * 1_024 * 1_024 * 1_024;
    const DISKO_SUFFIX: &'static str = "T";
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Size<U: SizeUnit> {
    pub value: u64,
    _unit: PhantomData<U>,
}

impl<U: SizeUnit> Size<U> {
    pub fn new(value: u64) -> Self {
        Self {
            value,
            _unit: PhantomData,
        }
    }

    pub fn to_bytes(self) -> u64 {
        self.value * U::BYTES
    }

    pub fn to_disko_str(self) -> String {
        format!("{}{}", self.value, U::DISKO_SUFFIX)
    }

    pub fn convert<V: SizeUnit>(self) -> Size<V> {
        Size::new(self.to_bytes() / V::BYTES)
    }
}

/// Partition size: a concrete typed value, a percentage, or an exact byte count with display string.
#[derive(Debug, Clone)]
pub enum PartitionSize {
    Gib(Size<GiB>),
    Mib(Size<MiB>),
    /// Percentage of total disk — disko renders `"N%"` (e.g. `"100%"` for fill-remaining).
    Percent(u8),
    /// Known byte count with a pre-computed disko display string (e.g. from lsblk).
    Exact {
        bytes: u64,
        display: String,
    },
}

impl PartitionSize {
    pub fn remaining() -> Self {
        PartitionSize::Percent(100)
    }

    pub fn to_disko_str(&self) -> String {
        match self {
            PartitionSize::Gib(s) => s.to_disko_str(),
            PartitionSize::Mib(s) => s.to_disko_str(),
            PartitionSize::Percent(p) => format!("{}%", p),
            PartitionSize::Exact { display, .. } => display.clone(),
        }
    }

    pub fn to_bytes(&self, disk_total: u64) -> u64 {
        match self {
            PartitionSize::Gib(s) => s.to_bytes(),
            PartitionSize::Mib(s) => s.to_bytes(),
            PartitionSize::Percent(p) => disk_total * (*p as u64) / 100,
            PartitionSize::Exact { bytes, .. } => *bytes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size_to_bytes() {
        assert_eq!(Size::<GiB>::new(2).to_bytes(), 2 * 1_024 * 1_024 * 1_024);
    }

    #[test]
    fn size_to_disko_str() {
        assert_eq!(Size::<GiB>::new(2).to_disko_str(), "2G");
        assert_eq!(Size::<MiB>::new(512).to_disko_str(), "512M");
        assert_eq!(Size::<TiB>::new(1).to_disko_str(), "1T");
    }

    #[test]
    fn size_convert() {
        let mib: Size<MiB> = Size::<GiB>::new(2).convert();
        assert_eq!(mib.value, 2048);
    }

    #[test]
    fn partition_size_disko_str() {
        assert_eq!(PartitionSize::remaining().to_disko_str(), "100%");
        assert_eq!(PartitionSize::Gib(Size::new(4)).to_disko_str(), "4G");
        assert_eq!(PartitionSize::Mib(Size::new(512)).to_disko_str(), "512M");
        assert_eq!(PartitionSize::Percent(50).to_disko_str(), "50%");
        assert_eq!(
            PartitionSize::Exact {
                bytes: 0,
                display: "42G".into()
            }
            .to_disko_str(),
            "42G"
        );
    }

    #[test]
    fn partition_size_to_bytes() {
        assert_eq!(PartitionSize::Percent(50).to_bytes(1000), 500);
        assert_eq!(
            PartitionSize::Exact {
                bytes: 999,
                display: String::new()
            }
            .to_bytes(0),
            999
        );
    }
}
