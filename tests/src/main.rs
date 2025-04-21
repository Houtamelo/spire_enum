#![allow(unused)]

mod advanced_enum_test;
mod basic_enum_test;
mod delegated_enum;
mod settings_enum;
mod state_machine_test;
mod weird;

use std::fmt::Debug;

use spire_enum_macros::{delegate_impl, delegated_enum};

fn main() {
    println!("Hello, world!");
}
