use std::fmt::Debug;

use super::*;

// Test with non-conflicting variant types using different wrapper types
#[variant_generic_table]
#[allow(unused)]
pub enum GenericEnum<T, E>
where
    T: Debug,
    E: Debug,
{
    Ok(Box<T>),
    Err(std::rc::Rc<E>), // Using Rc<E> to avoid conflict with Box<T>
    None(()),
}

#[test]
fn test_generic_enum_variant_generic_table() {
    // Create a table for the generic enum with concrete types
    let mut table = GenericEnumVariantGenericTable::<String>::new(
        "result".to_string(),
        "result".to_string(),
        "result".to_string(),
    );

    // Access values by concrete types
    let t_value: &String = table.get::<Box<String>>();
    assert_eq!(*t_value, "result");

    let e_value: &String = table.get::<std::rc::Rc<String>>();
    assert_eq!(*e_value, "result");

    // Modify values
    table.get_mut::<Box<String>>().push_str(" modified");
    table.get_mut::<std::rc::Rc<String>>().push_str(" error");

    // Verify modifications
    assert_eq!(*table.get::<Box<String>>(), "result modified");
    assert_eq!(*table.get::<std::rc::Rc<String>>(), "result error");

    // Test iteration
    let variants: Vec<generic_enum_variant_generic_table::GenericEnumOwn<String>> =
        table.into_iter().collect();
    assert_eq!(variants.len(), 3);

    // Verify the variants have the expected values
    let has_ok = variants.iter().any(|v| {
        match v {
            generic_enum_variant_generic_table::GenericEnumOwn::Ok(s) => *s == "result modified",
            _ => false,
        }
    });

    let has_err = variants.iter().any(|v| {
        match v {
            generic_enum_variant_generic_table::GenericEnumOwn::Err(s) => *s == "result error",
            _ => false,
        }
    });

    assert!(has_ok);
    assert!(has_err);
}

// Test with struct fields
#[variant_generic_table]
#[allow(unused)]
pub enum ConfigEnum {
    Setting(SettingConfig),
    User(UserConfig),
}

#[derive(Debug, Clone, PartialEq)]
#[allow(unused)]
pub struct SettingConfig {
    name:  String,
    value: i32,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(unused)]
pub struct UserConfig {
    username: String,
    admin: bool,
}

#[test]
fn test_struct_variant_generic_table() {
    // Create a table with complex value type
    #[derive(Clone, Debug)]
    struct ConfigValue {
        id:   usize,
        data: String,
    }

    let setting_value = ConfigValue {
        id:   1,
        data: "setting".to_string(),
    };
    let user_value = ConfigValue {
        id:   2,
        data: "user".to_string(),
    };

    let table = ConfigEnumVariantGenericTable::new(setting_value.clone(), user_value.clone());

    // Access by original struct types
    let setting_ref: &ConfigValue = table.get::<SettingConfig>();
    assert_eq!(setting_ref.id, 1);
    assert_eq!(setting_ref.data, "setting");

    let user_ref: &ConfigValue = table.get::<UserConfig>();
    assert_eq!(user_ref.id, 2);
    assert_eq!(user_ref.data, "user");

    // Convert to enum and verify
    let variants: Vec<config_enum_variant_generic_table::ConfigEnumOwn<ConfigValue>> =
        table.into_iter().collect();
    assert_eq!(variants.len(), 2);

    let has_setting = variants.iter().any(|v| {
        match v {
            config_enum_variant_generic_table::ConfigEnumOwn::Setting(s) => {
                s.id == 1 && s.data == "setting"
            }
            _ => false,
        }
    });

    let has_user = variants.iter().any(|v| {
        match v {
            config_enum_variant_generic_table::ConfigEnumOwn::User(u) => {
                u.id == 2 && u.data == "user"
            }
            _ => false,
        }
    });

    assert!(has_setting);
    assert!(has_user);
}

// Test with lifetime parameters
#[variant_generic_table]
#[allow(unused)]
pub enum LifetimeEnum<'a> {
    Borrowed(&'a String),
    Static(&'static str),
    None(()),
}

#[test]
fn test_lifetime_variant_generic_table() {
    let string = String::from("hello");

    // Create a table with references as generic value
    let table = LifetimeEnumVariantGenericTable::new(
        &string, // Reference value for all variants
        &string, &string,
    );

    // Access by the original types
    let borrowed_ref = table.get::<&str>();
    assert_eq!(**borrowed_ref, "hello");

    let static_ref = table.get::<&'static str>();
    assert_eq!(**static_ref, "hello");

    // Test iteration
    for variant in table.iter() {
        match variant {
            lifetime_enum_variant_generic_table::LifetimeEnumRef::Borrowed(r) => {
                assert_eq!(**r, "hello");
            }
            lifetime_enum_variant_generic_table::LifetimeEnumRef::Static(r) => {
                assert_eq!(**r, "hello");
            }
            _ => {}
        }
    }
}

// Test using the macro with a real-world example similar to SettingsEnum
#[variant_generic_table]
#[allow(unused)]
pub enum AppSettings {
    WindowSize(i32),
    Fullscreen(bool),
    Volume(f32),
    Username(String),
}

#[test]
fn test_app_settings_variant_generic_table() {
    // Define a setting value that can store different types of settings
    #[derive(Clone, Debug, PartialEq)]
    struct SettingValue {
        key: String,
        value_type: String,
        modified: bool,
    }

    // Create a table mapping each setting type to a SettingValue
    let mut table = AppSettingsVariantGenericTable::new(
        SettingValue {
            key: "window_size".to_string(),
            value_type: "integer".to_string(),
            modified: false,
        },
        SettingValue {
            key: "fullscreen".to_string(),
            value_type: "boolean".to_string(),
            modified: false,
        },
        SettingValue {
            key: "volume".to_string(),
            value_type: "float".to_string(),
            modified: false,
        },
        SettingValue {
            key: "username".to_string(),
            value_type: "string".to_string(),
            modified: false,
        },
    );

    // Test accessing values
    let window_size: &SettingValue = table.get::<i32>();
    assert_eq!(window_size.key, "window_size");

    let fullscreen: &SettingValue = table.get::<bool>();
    assert_eq!(fullscreen.key, "fullscreen");

    // Modify a value
    table.get_mut::<bool>().modified = true;

    // Verify modification
    assert!(table.get::<bool>().modified);

    // Test creating a filled table
    let default_value = SettingValue {
        key: "default".to_string(),
        value_type: "unknown".to_string(),
        modified: false,
    };

    let filled_table = AppSettingsVariantGenericTable::filled_with(default_value.clone());

    // Verify all settings have the default value
    for variant in filled_table.iter() {
        match variant {
            app_settings_variant_generic_table::AppSettingsRef::WindowSize(v)
            | app_settings_variant_generic_table::AppSettingsRef::Fullscreen(v)
            | app_settings_variant_generic_table::AppSettingsRef::Volume(v)
            | app_settings_variant_generic_table::AppSettingsRef::Username(v) => {
                assert_eq!(*v, default_value);
            }
        }
    }
}

// Test with non-conflicting wrapper types for generic parameters
#[variant_generic_table]
#[allow(unused)]
pub enum DataContainer {
    Integer(i32),
    Float(f64),
    Boolean(bool),
    String(String),
    List(Vec<u8>),         // Using concrete type Vec<u8> instead of generic type
    Optional(Option<i32>), // Using concrete type Option<i32>
}

#[test]
fn test_comprehensive_variant_generic_table() {
    // Create a custom type for the generic value
    #[derive(Debug, Clone, PartialEq)]
    struct Record {
        id:   usize,
        data: String,
    }

    // Create records for each type
    let int_record = Record {
        id:   1,
        data: "integer".to_string(),
    };
    let float_record = Record {
        id:   2,
        data: "float".to_string(),
    };
    let bool_record = Record {
        id:   3,
        data: "boolean".to_string(),
    };
    let string_record = Record {
        id:   4,
        data: "string".to_string(),
    };
    let list_record = Record {
        id:   5,
        data: "list".to_string(),
    };
    let opt_record = Record {
        id:   6,
        data: "optional".to_string(),
    };

    // Create a table mapping each variant type to a record
    let mut table = DataContainerVariantGenericTable::new(
        int_record.clone(),
        float_record.clone(),
        bool_record.clone(),
        string_record.clone(),
        list_record.clone(),
        opt_record.clone(),
    );

    // Test accessing values by original types
    let int_value: &Record = table.get::<i32>();
    assert_eq!(int_value.id, 1);
    assert_eq!(int_value.data, "integer");

    let string_value: &Record = table.get::<String>();
    assert_eq!(string_value.id, 4);
    assert_eq!(string_value.data, "string");

    let list_value: &Record = table.get::<Vec<u8>>();
    assert_eq!(list_value.id, 5);
    assert_eq!(list_value.data, "list");

    // Modify values
    table.get_mut::<i32>().data = "modified integer".to_string();
    table.get_mut::<Vec<u8>>().data = "modified list".to_string();

    // Verify modifications
    assert_eq!(table.get::<i32>().data, "modified integer");
    assert_eq!(table.get::<Vec<u8>>().data, "modified list");

    // Test iteration
    let variants: Vec<data_container_variant_generic_table::DataContainerOwn<Record>> =
        table.into_iter().collect();

    assert_eq!(variants.len(), 6);

    // Verify specific variants
    let modified_int_record = Record {
        id:   1,
        data: "modified integer".to_string(),
    };
    let modified_list_record = Record {
        id:   5,
        data: "modified list".to_string(),
    };

    let has_int = variants.iter().any(|v| {
        match v {
            data_container_variant_generic_table::DataContainerOwn::Integer(r) => {
                r.id == modified_int_record.id && r.data == modified_int_record.data
            }
            _ => false,
        }
    });

    let has_list = variants.iter().any(|v| {
        match v {
            data_container_variant_generic_table::DataContainerOwn::List(r) => {
                r.id == modified_list_record.id && r.data == modified_list_record.data
            }
            _ => false,
        }
    });

    assert!(has_int);
    assert!(has_list);
}

// Test with multiple distinct type wrappers to avoid conflicts
#[variant_generic_table]
#[allow(unused)]
pub enum DistinctTypeEnum {
    BoxedInt(Box<i32>),
    RcString(std::rc::Rc<String>),
    VecBool(Vec<bool>),
    OptionFloat(Option<f64>),
}

#[test]
fn test_distinct_type_enum() {
    let table = DistinctTypeEnumVariantGenericTable::new(42, 42, 42, 42);

    // Access each type
    let boxed_int: &i32 = table.get::<Box<i32>>();
    assert_eq!(*boxed_int, 42);

    let rc_string: &i32 = table.get::<std::rc::Rc<String>>();
    assert_eq!(*rc_string, 42);

    let vec_bool: &i32 = table.get::<Vec<bool>>();
    assert_eq!(*vec_bool, 42);

    let opt_float: &i32 = table.get::<Option<f64>>();
    assert_eq!(*opt_float, 42);

    // Create table with string values
    let table = DistinctTypeEnumVariantGenericTable::new(
        "boxed".to_string(),
        "rc".to_string(),
        "vec".to_string(),
        "option".to_string(),
    );

    // Verify string values
    assert_eq!(*table.get::<Box<i32>>(), "boxed");
    assert_eq!(*table.get::<std::rc::Rc<String>>(), "rc");
    assert_eq!(*table.get::<Vec<bool>>(), "vec");
    assert_eq!(*table.get::<Option<f64>>(), "option");
}
