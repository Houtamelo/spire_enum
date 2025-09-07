#![allow(unused)]
#![allow(unused_qualifications)]

mod advanced_enum_test;
mod basic_enum_test;
mod conditional_compilation;
mod delegated_enum;
mod discriminant_generic_tables;
mod settings_enum;
mod state_machine_test;
mod variant_generic_tables;
mod variant_type_tables;
mod weird;

use std::fmt::Debug;

use spire_enum::prelude::*;

fn main() {
    println!("Hello, world!");
}
