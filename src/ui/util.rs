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

pub fn format_size(s: Size) -> String {
    let bytes = s.bytes();
    let tip = s
        .to_string()
        .split(" ")
        .nth(1)
        .unwrap_or_default()
        .to_string();

    let the = match tip.to_string().as_str() {
        "TiB" => 1024_u64.pow(4),
        "GiB" => 1024_u64.pow(3),
        "MiB" => 1024_u64.pow(2),
        "KiB" => 1024,
        _ => 0,
    };

    let s = format!("{}", bytes as f64 / the as f64).to_string();

    let main = match s.find(|x| x == '.') {
        Some(x) => s[..(x + 3)].to_string(),
        None => s,
    };
    format!("{} {tip}", main)
}
