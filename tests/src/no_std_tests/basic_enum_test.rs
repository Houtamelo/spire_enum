use core::time::Duration;

use super::*;

// Test with a simple enum with different variant types
#[delegated_enum]
pub enum Result<T, E> {
    Ok(T),
    Err(E),
}

#[delegate_impl]
impl<T: Clone, E: Clone> Clone for Result<T, E>
where
    Result<T, E>: From<T>,
    Result<T, E>: From<E>,
{
    fn clone(&self) -> Self {
        delegate_result! {
            self => |v| v.clone().into()
        }
    }
}

// Test with unit variants
#[delegated_enum]
pub enum Direction {
    North,
    South,
    East,
    West,
}

// Test with struct variants
#[delegated_enum]
pub enum Shape {
    Circle {
        radius: f64,
    },
    Rectangle {
        width:  f64,
        height: f64,
    },
    #[allow(unused)]
    Triangle {
        base:   f64,
        height: f64,
    },
}

// Test with tuple variants and generic parameters
#[delegated_enum(impl_conversions)]
pub enum Command<T> {
    Execute(Option<T>),
    Delay(Duration),
    Batch([T; 2]),
}

#[delegate_impl]
impl<T: Clone> Clone for Command<T> {
    fn clone(&self) -> Self;
}

#[cfg(test)]
mod tests {
    use super::*;

    impl From<i32> for Result<i32, ()> {
        fn from(value: i32) -> Self { Result::Ok(value) }
    }

    impl From<()> for Result<i32, ()> {
        fn from(value: ()) -> Self { Result::Err(value) }
    }

    #[test]
    fn test_result_delegation() {
        let ok = Result::Ok::<_, ()>(42);
        if let Result::Ok(42) = ok.clone() {
        } else {
            panic!("Clone did not work correctly");
        }

        let err = Result::Err::<i32, _>(());
        if let Result::Err(()) = err.clone() {
        } else {
            panic!("Clone did not work correctly");
        }
    }

    #[test]
    fn test_command_enum() {
        let cmd1 = Command::Execute(Some(42));
        if let Command::Execute(Some(42)) = cmd1.clone() {
        } else {
            panic!("Clone did not work correctly");
        }

        let cmd2: Command<()> = Command::Delay(Duration::from_secs(1));
        if let Command::Delay(dur) = cmd2.clone() {
            assert_eq!(dur.as_secs(), 1);
        } else {
            panic!("Clone did not work correctly");
        }

        let cmd3 = Command::Batch([44, 45]);
        if let Command::Batch(arr) = cmd3.clone() {
            assert_eq!(arr, [44, 45]);
        } else {
            panic!("Clone did not work correctly");
        }
    }
}
