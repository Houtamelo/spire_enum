use std::fmt::{Debug, Display};

use super::*;

// Test with custom settings
#[delegated_enum(extract_variants, impl_conversions)]
pub enum CustomOption<T> {
    CustomSome(T),
    CustomNone,
}

// Test with nested enum types
#[allow(unused)]
#[delegated_enum(impl_conversions)]
#[derive(Debug)]
pub enum Message {
    Text(String),
    Binary(Vec<u8>),
    Status(Status),
    Nested(Nested),
}

#[derive(Debug)]
pub struct Nested(pub Box<Message>);

#[derive(Debug, Clone)]
pub enum Status {
    Good,
    Bad,
}

// Test with complex type constraints
#[delegated_enum]
pub enum Either<L, R>
where
    L: Display,
    R: Debug,
{
    Left(L),
    Right(R),
}

// Test with lifetime parameters
#[delegated_enum]
pub enum Reference<'a, T> {
    Borrowed(&'a T),
    Owned(T),
}

// Test with valid supported settings
#[delegated_enum(extract_variants, impl_conversions)]
pub enum SupportedSettings {
    First(String),
    Second(i32),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::assert_matches;

    #[test]
    fn test_from_enum_for_enum() {
        let message = Message::Text(String::from("Hello"));
        let mut enum_ = FromEnum::<Message>::from_enum(message).unwrap();
        assert_matches!(enum_, Message::Text(_));

        let enum_ref = <Message as FromEnumRef<Message>>::from_enum_ref(&enum_);
        assert_matches!(enum_ref, Some(&Message::Text(_)));

        let enum_mut = FromEnumMut::<Message>::from_enum_mut(&mut enum_);
        assert_matches!(enum_mut, Some(&mut Message::Text(_)));
    }

    #[test]
    fn test_from_enum_for_variant() {
        let enum_ = Message::Text(String::from("Hello"));
        let var = <Nested as FromEnum<Message>>::from_enum(enum_);
        assert_matches!(var, Err(_));

        let mut enum_ = Message::Text(String::from("Hello"));
        let var_ref = <Nested as FromEnumRef<Message>>::from_enum_ref(&enum_);
        assert_matches!(var_ref, None);

        let var_mut = <Nested as FromEnumMut<Message>>::from_enum_mut(&mut enum_);
        assert_matches!(var_mut, None);
    }

    #[test]
    fn test_from_enum_for_wrong_variant() {
        let enum_ = Message::Text(String::from("Hello"));
        let var = <String as FromEnum<Message>>::from_enum(enum_);
        assert_eq!(var.unwrap(), String::from("Hello"));

        let mut enum_ = Message::Text(String::from("Hello"));
        let var_ref = <String as FromEnumRef<Message>>::from_enum_ref(&enum_);
        assert_eq!(var_ref, Some(&String::from("Hello")));

        let var_mut = <String as FromEnumMut<Message>>::from_enum_mut(&mut enum_);
        assert_eq!(var_mut, Some(&mut String::from("Hello")));
    }

    #[test]
    fn test_option_delegation() {
        let some = CustomOption::CustomSome(CustomSome(42));
        let none = CustomOption::<i32>::CustomNone(CustomNone);

        match some {
            CustomOption::CustomSome(val) => assert_eq!(val.0, 42),
            CustomOption::CustomNone(_) => panic!("Expected Some variant"),
        }

        assert!(matches!(none, CustomOption::CustomNone(_)));
    }

    #[test]
    fn test_message_delegation() {
        #[allow(unused)]
        let text = Message::Text("Hello".to_string());
        #[allow(unused)]
        let binary = Message::Binary(vec![1, 2, 3]);
        #[allow(unused)]
        let status = Message::Status(Status::Good);
        let nested = Message::Nested(Nested(Box::new(Message::Text("Nested".to_string()))));

        match nested {
            Message::Nested(nested) => match *nested.0 {
                Message::Text(ref s) => assert_eq!(s, "Nested"),
                _ => panic!("Expected Text variant inside Nested"),
            },
            _ => panic!("Expected Nested variant"),
        }
    }

    #[test]
    fn test_either_delegation() {
        let left = Either::<String, i32>::Left("left".to_string());
        let right = Either::<String, i32>::Right(42);

        match left {
            Either::Left(s) => assert_eq!(s, "left"),
            Either::Right(_) => panic!("Expected Left variant"),
        }

        match right {
            Either::Left(_) => panic!("Expected Right variant"),
            Either::Right(i) => assert_eq!(i, 42),
        }
    }

    #[test]
    fn test_reference_delegation() {
        let value = 42;
        let borrowed = Reference::Borrowed(&value);
        let owned = Reference::Owned(43);

        match borrowed {
            Reference::Borrowed(v) => assert_eq!(*v, 42),
            Reference::Owned(_) => panic!("Expected Borrowed variant"),
        }

        match owned {
            Reference::Borrowed(_) => panic!("Expected Owned variant"),
            Reference::Owned(v) => assert_eq!(v, 43),
        }
    }

    #[test]
    fn test_supported_settings() {
        // Testing with valid supported settings
        let first = SupportedSettings::First(First("test".to_string()));
        let second = SupportedSettings::Second(Second(42));

        match first {
            SupportedSettings::First(s) => assert_eq!(s.0, "test"),
            _ => panic!("Expected First variant"),
        }

        match second {
            SupportedSettings::Second(i) => assert_eq!(i.0, 42),
            _ => panic!("Expected Second variant"),
        }
    }
}
