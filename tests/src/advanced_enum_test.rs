use std::fmt::{self, Debug, Display};

use super::*;

// Test with custom settings
#[delegated_enum(extract_variants, impl_conversions)]
pub enum Option<T> {
    Some(T),
    None,
}

// Test with nested enum types
#[delegated_enum]
pub enum Message {
    Text(String),
    Binary(Vec<u8>),
    Status { code: u16, message: String },
    Nested(Box<Message>),
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

// Test with attributes
#[delegated_enum]
#[derive(Debug, Clone)]
pub enum HttpMethod {
    #[allow(dead_code)]
    GET,
    #[allow(dead_code)]
    POST,
    #[allow(dead_code)]
    PUT,
    #[allow(dead_code)]
    DELETE,
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

    #[test]
    fn test_option_delegation() {
        let some = Option::Some(Some(42));
        let none = Option::<i32>::None(None);

        match some {
            Option::Some(val) => assert_eq!(val.0, 42),
            Option::None(_) => panic!("Expected Some variant"),
        }

        assert!(matches!(none, Option::None(..)));
    }

    #[test]
    fn test_message_delegation() {
        let text = Message::Text("Hello".to_string());
        let binary = Message::Binary(vec![1, 2, 3]);
        let status = Message::Status {
            code: 200,
            message: "OK".to_string(),
        };
        let nested = Message::Nested(Box::new(Message::Text("Nested".to_string())));

        match nested {
            Message::Nested(boxed) => {
                match *boxed {
                    Message::Text(ref s) => assert_eq!(s, "Nested"),
                    _ => panic!("Expected Text variant inside Nested"),
                }
            }
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
    fn test_http_method_delegation() {
        let method = HttpMethod::GET;
        assert!(matches!(method, HttpMethod::GET));

        // Test that Debug was properly derived
        assert_eq!(format!("{:?}", method), "GET");

        // Test that Clone was properly derived
        let cloned = method.clone();
        assert!(matches!(cloned, HttpMethod::GET));
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
