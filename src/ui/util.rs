#[derive(Debug, Clone, Copy)]
pub enum SizeType {
    TB,
    GB,
    MB,
    KB,
}

pub fn get_byte_from(x: SizeType) -> u64 {
    match x {
        SizeType::TB => 1024 ^ 4,
        SizeType::GB => 1024 ^ 3,
        SizeType::MB => 1024 ^ 2,
        SizeType::KB => 1024,
    }
}
