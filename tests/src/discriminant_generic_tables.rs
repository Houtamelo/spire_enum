use std::fmt::Debug;

use super::*;

// Define a basic enum with the discriminant_generic_table macro
#[discriminant_generic_table]
pub enum TestDiscriminants {
	First,
	Second,
	Third,
}

// Test with explicit discriminant values
#[discriminant_generic_table(
    derive(Clone),
    ty_name = CustomTable,
    mod_name = custom_table,
    attrs(derive(Debug)),
)]
#[repr(i32)]
pub enum ExplicitDiscriminants {
	First = 5,
	Second = 10,
	Third = 13,
}

#[test]
fn test() {
	let table = custom_table::CustomTable::new(5, 10, 3);

	// Ensure clone works
	let table_clone: CustomTable<i32> = table.clone();
	assert_eq!(table_clone[ExplicitDiscriminants::First], 5);
	assert_eq!(table_clone[ExplicitDiscriminants::Second], 10);
	assert_eq!(table_clone[ExplicitDiscriminants::Third], 3);

	// Ensure debug is implemented
	let _ = format!("{table:?}");

	// Ensure discriminant values match
	assert_eq!(ExplicitDiscriminants::First as i32, 5);
	assert_eq!(ExplicitDiscriminants::Second as i32, 10);
	assert_eq!(ExplicitDiscriminants::Third as i32, 13);
}
