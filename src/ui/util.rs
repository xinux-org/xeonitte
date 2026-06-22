use size::Size;

#[derive(Debug, Clone, Copy)]
pub enum SizeType {
    TB,
    GB,
    MB,
    KB,
}

pub fn represent(x: SizeType, y: u64) -> Size {
    let f = match x {
        SizeType::TB => Size::from_tib,
        SizeType::GB => Size::from_gib,
        SizeType::MB => Size::from_mib,
        SizeType::KB => Size::from_kib,
    };

    f(y)
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
