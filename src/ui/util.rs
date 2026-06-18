#[derive(Debug, Clone, Copy)]
pub enum SizeType {
    TB,
    GB,
    MB,
    KB,
}

pub fn get_byte_from(x: SizeType) -> u64 {
    match x {
        SizeType::TB => 1_000_000_000_000,
        SizeType::GB => 1_000_000_000,
        SizeType::MB => 1_000_000,
        SizeType::KB => 1_000,
    }
}
