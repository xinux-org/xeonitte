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
        // SizeType::TB => Size::from_tib,
        // SizeType::GB => Size::from_gib,
        // SizeType::MB => Size::from_mib,
        // SizeType::KB => Size::from_kib,
        SizeType::TB => "TiB",
        SizeType::GB => "GiB",
        SizeType::MB => "MiB",
        SizeType::KB => "KiB",
    };

    Size::from_str(&format!("{} {}", y, f)).unwrap_or_default()
    // f(y)
}

pub fn format_size(s: Size) -> String {
    let bytes = s.bytes();
    let size = s
        .to_string()
        .split(" ")
        .nth(0)
        .unwrap_or_default()
        .to_string();
    let tip = s
        .to_string()
        .split(" ")
        .nth(1)
        .unwrap_or_default()
        .to_string();
    let size_from_str = Size::from_str(&s.to_string());

    let significant = size
        .to_string()
        .split(".")
        .nth(0)
        .unwrap_or_default()
        .to_string();
    let exponent = size
        .to_string()
        .split(".")
        .nth(1)
        .unwrap_or_default()
        .parse::<u32>();

    // Size::from_str(&format!("{significant} {tip}")).unwrap_or_default()

    let the = match tip.to_string().as_str() {
        "TiB" => 1024_u64.pow(4),
        "GiB" => 1024_u64.pow(3),
        "MiB" => 1024_u64.pow(2),
        "KiB" => 1024,
        _ => 0,
    };

    println!("THEEEEEEEEEEEEEEEEEEEEEEEEEE: {the:?},        tip: {tip}");

    let s = format!("{}", bytes as f64 / the as f64).to_string();
    // let main = &s[..significant.len()];
    format!(
        "{} {tip}",
        s[..(s.find(|x| x == '.').unwrap_or(s.len() - 3) + 3)].to_string()
    )
    // s[..(s.find(|x| x == '.').unwrap_or_default() + 2)].to_string()

    // unimplemented!()
}

// pub fn get_byte_from(x: SizeType) -> u64 {
//     match x {
//         SizeType::TB => 1024 ^ 4,
//         SizeType::GB => 1024 ^ 3,
//         SizeType::MB => 1024 ^ 2,
//         SizeType::KB => 1024,
//     }
// }

// pub struct NewPartitionSize {
//     bytes: u64,
//     type_closure: Box<dyn FnOnce(u64) -> Size>,
// }

// impl std::fmt::Debug for NewPartitionSize {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("NewPartitionSize")
//             .field("bytes", &self.bytes)
//             .finish()
//     }
// }

// impl Default for NewPartitionSize {
//     fn default() -> Self {
//         Self {
//             bytes: Default::default(),
//             size_constructor: Box::new(Size::from_bytes::<u64>),
//         }
//     }
// }
