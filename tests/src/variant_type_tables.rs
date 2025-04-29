use std::{fmt::Debug, marker::PhantomData, rc::Rc};

use super::*;

#[delegated_enum(extract_variants(derive(Debug, Clone, Default, PartialEq)))]
#[variant_type_table(
    mod_name = custom_table,
    ty_name = SettingsTable,
    attrs(derive(Debug, Default)),
    derive(Clone),
)]
pub enum SettingsEnum {
	MaxFps(i32),
	DialogueTextSpeed(i32),
	Vsync(bool),
	MainVolume(i32),
	MusicVolume(i32),
	SfxVolume(i32),
	VoiceVolume(i32),
}

#[test]
fn test() {
	let mut table = custom_table::SettingsTable::default();
	*table.get_mut::<MaxFps>() = MaxFps(60);
	*table.get_mut::<MainVolume>() = MainVolume(10);
	*table.get_mut::<Vsync>() = Vsync(false);

	assert_eq!(*table.get::<MaxFps>(), MaxFps(60));
	assert_eq!(*table.get::<MainVolume>(), MainVolume(10));
	assert_eq!(*table.get::<Vsync>(), Vsync(false));

	// Ensure clone works
	let table_clone = table.clone();
	assert_eq!(*table_clone.get::<MaxFps>(), MaxFps(60));
	assert_eq!(*table_clone.get::<MainVolume>(), MainVolume(10));
	assert_eq!(*table_clone.get::<Vsync>(), Vsync(false));

	// Ensure debug is implemented.
	let _ = format!("{table:?}");
}

// Test with generic parameters
#[variant_type_table]
pub enum GenericEnum<T, E>
where T: Debug
{
	Ok(Rc<T>),
	Err(Box<E>),
	None(()),
}

#[allow(unused_qualifications)]
#[test]
fn test_generic_variant_type_table() {
	// Create a table with generic types
	let mut table = generic_enum_variant_type_table::GenericEnumVariantTypeTable::new(
		"success".to_string().into(), // T type for Ok variant
		42i32.into(),                 // E type for Err variant
		(),                           // Unit type for None variant
	);

	// Access the values with their actual types
	let ok_val: &String = table.get::<Rc<String>>();
	assert_eq!(ok_val, "success");

	let err_val: &i32 = table.get::<Box<i32>>();
	assert_eq!(*err_val, 42);

	// Mutate the values
	let err_val_mut: &mut i32 = table.get_mut::<Box<i32>>();
	*err_val_mut = 100;

	// Verify the mutation worked
	assert_eq!(**table.get::<Box<i32>>(), 100);

	// Convert to enum variants and collect them
	let variants: Vec<GenericEnum<String, i32>> = table.into_iter().collect();
	assert_eq!(variants.len(), 3);

	// Check specific variants
	let has_ok = variants.iter().any(|v| {
		match v {
			GenericEnum::Ok(val) => **val == "success",
			_ => false,
		}
	});

	let has_err = variants.iter().any(|v| {
		match v {
			GenericEnum::Err(val) => **val == 100,
			_ => false,
		}
	});

	let has_none = variants.iter().any(|v| matches!(v, GenericEnum::None(_)));

	assert!(has_ok);
	assert!(has_err);
	assert!(has_none);
}

// Test with single-field struct variants
#[variant_type_table]
pub enum ComplexEnum {
	Config(ConfigType),
	Status(StatusInfo),
	Empty(()),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConfigType {
	pub name:  String,
	pub value: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StatusInfo {
	pub code: u16,
	pub message: String,
}

#[test]
fn test_complex_struct_variant_type_table() {
	let config = ConfigType {
		name:  "setting".to_string(),
		value: 42,
	};

	let status = StatusInfo {
		code: 200,
		message: "OK".to_string(),
	};

	let mut table = complex_enum_variant_type_table::ComplexEnumVariantTypeTable::new(
		config.clone(),
		status.clone(),
		(),
	);

	// Test struct access
	let config_ref: &ConfigType = table.get::<ConfigType>();
	assert_eq!(config_ref.name, "setting");
	assert_eq!(config_ref.value, 42);

	// Modify the struct
	let status_ref: &mut StatusInfo = table.get_mut::<StatusInfo>();
	status_ref.message = "Updated".to_string();

	// Verify the update
	let status_ref: &StatusInfo = table.get::<StatusInfo>();
	assert_eq!(status_ref.message, "Updated");

	// Test iteration with references
	for variant in table.iter() {
		match variant {
			complex_enum_variant_type_table::ComplexEnumRef::Config(cfg) => {
				assert_eq!(cfg.name, "setting");
				assert_eq!(cfg.value, 42);
			}
			complex_enum_variant_type_table::ComplexEnumRef::Status(st) => {
				assert_eq!(st.message, "Updated");
				assert_eq!(st.code, 200);
			}
			complex_enum_variant_type_table::ComplexEnumRef::Empty(_) => {}
		}
	}

	// Convert to enum and check values
	let variants: Vec<ComplexEnum> = table.into_iter().collect();
	assert_eq!(variants.len(), 3);

	let has_config = variants.iter().any(|v| {
		match v {
			ComplexEnum::Config(cfg) => cfg == &config,
			_ => false,
		}
	});

	let modified_status = StatusInfo {
		code: 200,
		message: "Updated".to_string(),
	};

	let has_status = variants.iter().any(|v| {
		match v {
			ComplexEnum::Status(st) => st == &modified_status,
			_ => false,
		}
	});

	assert!(has_config);
	assert!(has_status);
}

// Test with lifetime parameters
#[variant_type_table]
pub enum LifetimeEnum<'a> {
	StaticRef(&'static str),
	Owned(&'a String),
	None(()),
}

#[test]
fn test_lifetime_variant_type_table() {
	let value = String::from("static string");

	// Create the table
	let mut table = lifetime_enum_variant_type_table::LifetimeEnumVariantTypeTable::new(
		"hello", // &'static str
		&value,  // &'a String
		(),      // Unit type
	);

	// Test access
	let static_ref: &&'static str = table.get::<&'static str>();
	assert_eq!(*static_ref, "hello");

	let owned = table.get::<&String>();
	assert_eq!(*owned, "static string");

	// Convert to enum variants
	let variants: Vec<LifetimeEnum> = table.into_iter().collect();
	assert_eq!(variants.len(), 3);

	let has_static_ref = variants.iter().any(|v| {
		match v {
			LifetimeEnum::StaticRef(s) => *s == "hello",
			_ => false,
		}
	});

	let has_owned = variants.iter().any(|v| {
		match v {
			LifetimeEnum::Owned(s) => *s == "static string",
			_ => false,
		}
	});

	assert!(has_static_ref);
	assert!(has_owned);
}

// Test with zero-sized types and unit variants
#[variant_type_table]
pub enum ZeroSizedEnum {
	Unit(()),
	Marker(PhantomData<i32>),
	Never(Option<()>),
}

#[test]
fn test_zero_sized_variant_type_table() {
	// Create the table with zero-sized types
	let table = zero_sized_enum_variant_type_table::ZeroSizedEnumVariantTypeTable::new(
		(),
		PhantomData::<i32>,
		None,
	);

	// Since these are zero-sized types, we can still verify the structure works
	let _unit_ref: &() = table.get::<()>();
	let _phantom_ref: &PhantomData<i32> = table.get::<PhantomData<i32>>();
	let _never_ref: &Option<()> = table.get::<Option<()>>();

	// Convert to enum and check variants exist
	let variants: Vec<ZeroSizedEnum> = table.into_iter().collect();
	assert_eq!(variants.len(), 3);

	assert!(variants.iter().any(|v| matches!(v, ZeroSizedEnum::Unit(_))));
	assert!(
		variants
			.iter()
			.any(|v| matches!(v, ZeroSizedEnum::Marker(_)))
	);
	assert!(
		variants
			.iter()
			.any(|v| matches!(v, ZeroSizedEnum::Never(_)))
	);
}

// Test with multiple generics and where clauses
#[variant_type_table]
pub enum MultiGenericEnum<T, E>
where
	T: Clone,
	E: Debug,
{
	First(Box<T>),
	Second(Rc<E>),
	Third(String),
}

#[test]
fn test_multi_generic_variant_type_table() {
	// Create a table with multiple generic types
	let mut table = multi_generic_enum_variant_type_table::MultiGenericEnumVariantTypeTable::new(
		42.into(),                  // T type
		"error".to_string().into(), // E type
		"third".to_string(),        // String type
	);

	// Access the values
	let first_val: &i32 = table.get::<Box<i32>>();
	assert_eq!(*first_val, 42);

	let second_val: &String = table.get::<String>();
	assert_eq!(*second_val, "third");

	assert_eq!(**table.get::<Rc<String>>(), "error");

	// Note: We have two String fields, so we can't directly test the third one
	// The macro uses types for lookup, not field positions

	// Modify a value
	*table.get_mut::<Box<i32>>() = 100.into();

	// Verify modification
	assert_eq!(**table.get::<Box<i32>>(), 100);

	// Convert to enum and verify
	let variants: Vec<MultiGenericEnum<i32, String>> = table.into_iter().collect();
	assert_eq!(variants.len(), 3);

	let has_first = variants.iter().any(|v| {
		match v {
			MultiGenericEnum::First(val) => **val == 100,
			_ => false,
		}
	});

	let has_second = variants.iter().any(|v| {
		match v {
			MultiGenericEnum::Second(val) => **val == "error",
			_ => false,
		}
	});

	assert!(has_first);
	assert!(has_second);
}

// Test with trait implementations
#[variant_type_table]
pub enum TraitEnum {
	Value(ValueType),
	Option(OptionType),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ValueType {
	id:   u32,
	name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OptionType {
	enabled: bool,
}

// Implement custom trait for the variant types
pub trait Configurable {
	fn is_enabled(&self) -> bool;
	fn get_id(&self) -> Option<u32>;
}

impl Configurable for ValueType {
	fn is_enabled(&self) -> bool { true }

	fn get_id(&self) -> Option<u32> { Some(self.id) }
}

impl Configurable for OptionType {
	fn is_enabled(&self) -> bool { self.enabled }

	fn get_id(&self) -> Option<u32> { None }
}

#[test]
fn test_trait_integration() {
	let value = ValueType {
		id:   123,
		name: "Test".to_string(),
	};

	let option = OptionType { enabled: true };

	let mut table = trait_enum_variant_type_table::TraitEnumVariantTypeTable::new(
		value.clone(),
		option.clone(),
	);

	// Test using trait methods on the types in the table
	let value_ref: &ValueType = table.get::<ValueType>();
	assert_eq!(value_ref.get_id(), Some(123));
	assert!(value_ref.is_enabled());

	let option_ref: &OptionType = table.get::<OptionType>();
	assert_eq!(option_ref.get_id(), None);
	assert!(option_ref.is_enabled());

	// Modify through the table
	let option_mut: &mut OptionType = table.get_mut::<OptionType>();
	option_mut.enabled = false;

	// Verify modification through trait method
	let option_ref: &OptionType = table.get::<OptionType>();
	assert!(!option_ref.is_enabled());

	// Convert to enum and check values
	let variants: Vec<TraitEnum> = table.into_iter().collect();
	assert_eq!(variants.len(), 2);

	let has_value = variants.iter().any(|v| {
		match v {
			TraitEnum::Value(val) => val == &value,
			_ => false,
		}
	});

	let modified_option = OptionType { enabled: false };
	let has_option = variants.iter().any(|v| {
		match v {
			TraitEnum::Option(opt) => opt == &modified_option,
			_ => false,
		}
	});

	assert!(has_value);
	assert!(has_option);
}

// Test with a combination of different types
#[variant_type_table]
pub enum MixedEnum {
	Integer(i32),
	Float(f64),
	Text(String),
	Flag(bool),
	Complex(ComplexData),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ComplexData {
	id: u32,
	value: String,
	enabled: bool,
}

#[test]
fn test_mixed_variant_type_table() {
	let complex = ComplexData {
		id: 42,
		value: "test".to_string(),
		enabled: true,
	};

	// Create the table
	let mut table = mixed_enum_variant_type_table::MixedEnumVariantTypeTable::new(
		10,
		3.13,
		"hello".to_string(),
		true,
		complex.clone(),
	);

	// Test access to different types
	assert_eq!(*table.get::<i32>(), 10);
	assert_eq!(*table.get::<f64>(), 3.13);
	assert_eq!(*table.get::<String>(), "hello");
	assert!(*table.get::<bool>());
	assert_eq!(*table.get::<ComplexData>(), complex);

	// Modify multiple values
	*table.get_mut::<i32>() = 20;
	*table.get_mut::<String>() = "updated".to_string();
	*table.get_mut::<bool>() = false;

	// Verify modifications
	assert_eq!(*table.get::<i32>(), 20);
	assert_eq!(*table.get::<String>(), "updated");
	assert!(!*table.get::<bool>());

	// Convert to enum and check
	let variants: Vec<MixedEnum> = table.into_iter().collect();
	assert_eq!(variants.len(), 5);

	let has_integer = variants.iter().any(|v| matches!(v, MixedEnum::Integer(20)));
	let has_float = variants.iter().any(|v| matches!(v, MixedEnum::Float(3.13)));
	let has_text = variants.iter().any(|v| {
		match v {
			MixedEnum::Text(s) => s == "updated",
			_ => false,
		}
	});
	let has_flag = variants.iter().any(|v| matches!(v, MixedEnum::Flag(false)));

	assert!(has_integer);
	assert!(has_float);
	assert!(has_text);
	assert!(has_flag);
}

// Test with a more complex example combining multiple features

#[variant_type_table]
pub enum ConfigEnum<'a, T>
where T: Debug + 'a
{
	Simple(i32),
	Text(String),
	Reference(&'a T),
	Boxed(Box<T>),
}

#[test]
fn test_comprehensive_variant_type_table() {
	let data = String::from("data");
	let boxed_value = Box::new("boxed value".to_string());

	// Create the table
	let mut table = config_enum_variant_type_table::ConfigEnumVariantTypeTable::new(
		10,
		"text".to_string(),
		&data,
		boxed_value.clone(),
	);

	// Test access
	assert_eq!(*table.get::<i32>(), 10);
	assert_eq!(*table.get::<String>(), "text");
	assert_eq!(**table.get::<&String>(), "data");
	assert_eq!(**table.get::<Box<String>>(), "boxed value");

	// Modify some values
	*table.get_mut::<i32>() = 20;
	*table.get_mut::<String>() = "modified".to_string();

	// Verify modifications
	assert_eq!(*table.get::<i32>(), 20);
	assert_eq!(*table.get::<String>(), "modified");

	// Test iteration with references
	for variant in table.iter() {
		match variant {
			config_enum_variant_type_table::ConfigEnumRef::Simple(i) => {
				assert_eq!(*i, 20);
			}
			config_enum_variant_type_table::ConfigEnumRef::Text(s) => {
				assert_eq!(*s, "modified");
			}
			config_enum_variant_type_table::ConfigEnumRef::Reference(r) => {
				assert_eq!(*r, "data");
			}
			config_enum_variant_type_table::ConfigEnumRef::Boxed(b) => {
				assert_eq!(**b, "boxed value");
			}
		}
	}

	// Test converting to enum
	let variants: Vec<ConfigEnum<String>> = table.into_iter().collect();
	assert_eq!(variants.len(), 4);
}
