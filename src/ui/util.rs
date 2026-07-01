use size::Size;

#[derive(Debug, Clone, Copy)]
pub enum SizeType {
    TB,
    GB,
    MB,
    KB,
}

pub fn represent(x: SizeType, y: f64) -> Size {
    let f = match x {
        SizeType::TB => "TiB",
        SizeType::GB => "GiB",
        SizeType::MB => "MiB",
        SizeType::KB => "KiB",
    };

    Size::from_str(&format!("{} {}", y, f)).unwrap_or_default()
}
