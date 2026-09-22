pub mod disko;
pub mod i18n;
pub mod install;
pub mod language;
pub mod make_config;
pub mod parse;
pub mod report;

use size::Size;

#[derive(Debug, Clone, Copy)]
pub enum SizeType {
    TB,
    GB,
    MB,
    KB,
}
impl From<SizeType> for String {
    fn from(value: SizeType) -> Self {
        match value {
            SizeType::TB => "TiB",
            SizeType::GB => "GiB",
            SizeType::MB => "MiB",
            SizeType::KB => "KiB",
        }
        .to_string()
    }
}

impl From<String> for SizeType {
    fn from(value: String) -> Self {
        match value.to_lowercase().as_str() {
            "tib" => SizeType::TB,
            "gib" => SizeType::GB,
            "mib" => SizeType::MB,
            _ => SizeType::KB,
        }
    }
}

pub fn represent(size_type: SizeType, bytes: f64) -> Size {
    Size::from_str(&format!("{} {}", bytes, String::from(size_type))).unwrap_or_default()
}
