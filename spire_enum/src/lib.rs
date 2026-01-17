#![doc = include_str!("../../README.md")]
#![no_std]

mod traits;

pub mod prelude {
    pub use spire_enum_macros::{
        delegate_impl,
        delegated_enum,
        discriminant_generic_table,
        variant_generic_table,
        variant_type_table,
    };

    pub use crate::traits::*;
}
