use super::*;

#[delegated_enum(extract_variants(derive(Debug, Clone), derive(Default)))]
#[variant_generic_table(
    ty_name = ActorStats,
    mod_name = actor_stats_table,
    attrs(derive(Debug), derive(Clone)),
    derive(Default),
)]
enum Stat {
    Strength,
    Intelligence,
    Constitution,
}

#[test]
fn test() {
    let mut table = ActorStats::new(5, 10, 7);

    // Ensure clone works
    #[allow(unused)]
    let table_clone = table.clone();

    assert_eq!(table.get::<Strength>(), &5);
    assert_eq!(table.get::<Intelligence>(), &10);
    assert_eq!(table.get::<Constitution>(), &7);

    // Ensure debug is implemented
    let _ = <ActorStats<i32> as Debug>::fmt;

    // Ensure default is implemented
    let _ = ActorStats::<f64>::default();

    table.set::<Strength>(8);
    assert_eq!(table.get::<Strength>(), &8);
}

// Define an enum with the variant_generic_table macro
#[variant_generic_table]
#[allow(unused)]
pub enum TestEnum {
    Number(i32),
    Text(&'static str),
    Flag(bool),
}

#[test]
fn test_variant_generic_table_basic() {
    // Create a table with a single generic type
    let mut table = TestEnumVariantGenericTable::new(
        42, // Generic value for Number variant
        42, // Generic value for Text variant
        42, // Generic value for Flag variant
    );

    // Test accessing the values by their original types
    let number_value: &i32 = table.get::<i32>();
    assert_eq!(*number_value, 42);

    // Modify a value
    *table.get_mut::<i32>() = 100;

    // Verify the modification
    assert_eq!(*table.get::<i32>(), 100);

    // Test iteration with references
    for variant in table.iter() {
        match variant {
            test_enum_variant_generic_table::TestEnumRef::Number(n) => {
                assert_eq!(*n, 100);
            }
            test_enum_variant_generic_table::TestEnumRef::Text(t) => {
                assert_eq!(*t, 42);
            }
            test_enum_variant_generic_table::TestEnumRef::Flag(f) => {
                assert_eq!(*f, 42);
            }
        }
    }

    let mut iter = table.into_iter();
    // Test conversion to owned enum variants
    let variants = [
        iter.next().unwrap(),
        iter.next().unwrap(),
        iter.next().unwrap(),
    ];
    assert_eq!(variants.len(), 3);
    assert!(iter.next().is_none());

    // Verify correct variant types
    let has_number = variants.iter().any(|v| {
        match v {
            test_enum_variant_generic_table::TestEnumOwn::Number(n) => *n == 100,
            _ => false,
        }
    });

    let has_text = variants.iter().any(|v| {
        match v {
            test_enum_variant_generic_table::TestEnumOwn::Text(t) => *t == 42,
            _ => false,
        }
    });

    let has_flag = variants.iter().any(|v| {
        match v {
            test_enum_variant_generic_table::TestEnumOwn::Flag(f) => *f == 42,
            _ => false,
        }
    });

    assert!(has_number);
    assert!(has_text);
    assert!(has_flag);
}

#[test]
#[cfg(not(feature = "no_std"))]
fn test_variant_generic_table_string() {
    // Create a table with String as the generic type
    let mut table = TestEnumVariantGenericTable::new(
        "number".to_string(), // Generic String for Number variant
        "text".to_string(),   // Generic String for Text variant
        "flag".to_string(),   // Generic String for Flag variant
    );

    // Test accessing values by original types
    let string_value: &String = table.get::<&'static str>();
    assert_eq!(*string_value, "text");

    // Modify a value
    table.get_mut::<&'static str>().push_str(" modified");

    // Verify multiple string values through iteration
    for variant in table.iter() {
        if let test_enum_variant_generic_table::TestEnumRef::Text(t) = variant {
            assert_eq!(*t, "text modified");
        }
    }
}

#[test]
fn test_variant_generic_table_filled() {
    // Test the filled constructor that creates a table with all the same value
    let table = TestEnumVariantGenericTable::filled_with(42);

    // Verify all values are the same
    assert_eq!(*table.get::<i32>(), 42);
    assert_eq!(*table.get::<&'static str>(), 42);
    assert_eq!(*table.get::<bool>(), 42);

    // Create with string and verify
    let table = TestEnumVariantGenericTable::filled_with("same");

    assert_eq!(*table.get::<i32>(), "same");
    assert_eq!(*table.get::<&'static str>(), "same");
    assert_eq!(*table.get::<bool>(), "same");
}

// Test trait implementation for generic values
#[allow(unused)]
pub trait Identifiable {
    fn id(&self) -> usize;
}

#[derive(Clone, Debug)]
#[allow(unused)]
struct IdentifiableValue {
    id: usize,
    value: &'static str,
}

impl Identifiable for IdentifiableValue {
    fn id(&self) -> usize { self.id }
}

#[test]
fn test_variant_generic_table_with_trait() {
    // Create values that implement a trait
    let val1 = IdentifiableValue {
        id: 1,
        value: "number",
    };
    let val2 = IdentifiableValue {
        id: 2,
        value: "text",
    };
    let val3 = IdentifiableValue {
        id: 3,
        value: "flag",
    };

    // Create a table with these values
    let table = TestEnumVariantGenericTable::new(val1.clone(), val2.clone(), val3.clone());

    // Access values by original types
    let number_value: &IdentifiableValue = table.get::<i32>();
    assert_eq!(number_value.id(), 1);

    let text_value: &IdentifiableValue = table.get::<&'static str>();
    assert_eq!(text_value.id(), 2);

    let flag_value: &IdentifiableValue = table.get::<bool>();
    assert_eq!(flag_value.id(), 3);

    // Test using the trait method through iteration
    for variant in table.iter() {
        match variant {
            test_enum_variant_generic_table::TestEnumRef::Number(n) => {
                assert_eq!(n.id(), 1);
            }
            test_enum_variant_generic_table::TestEnumRef::Text(t) => {
                assert_eq!(t.id(), 2);
            }
            test_enum_variant_generic_table::TestEnumRef::Flag(f) => {
                assert_eq!(f.id(), 3);
            }
        }
    }
}
