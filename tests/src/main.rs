#![cfg_attr(feature = "no_std", no_std)]
mod no_std_tests;

#[cfg(not(feature = "no_std"))]
mod std_tests;

use core::fmt::Debug;

use spire_enum::prelude::*;

fn main() {}
