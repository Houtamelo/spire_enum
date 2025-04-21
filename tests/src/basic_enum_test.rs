use super::*;

// Test with a simple enum with different variant types
#[delegated_enum]
pub enum Result<T, E> {
	Ok(T),
	Err(E),
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
	Circle { radius: f64 },
	Rectangle { width: f64, height: f64 },
	Triangle { base: f64, height: f64 },
}

// Test with tuple variants and generic parameters
#[delegated_enum]
pub enum Command<T> {
	Execute(T),
	Delay(std::time::Duration, T),
	Batch(Vec<T>),
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_result_delegation() {
		let ok = Result::Ok::<_, ()>(42);
		let err = Result::Err::<(), _>("error");

		match ok {
			Result::Ok(value) => assert_eq!(value, 42),
			Result::Err(_) => panic!("Expected Ok variant"),
		}

		match err {
			Result::Ok(_) => panic!("Expected Err variant"),
			Result::Err(e) => assert_eq!(e, "error"),
		}
	}

	#[test]
	fn test_direction_enum() {
		let dir = Direction::North;
		assert!(matches!(dir, Direction::North));

		let dirs = vec![
			Direction::North,
			Direction::South,
			Direction::East,
			Direction::West,
		];
		assert_eq!(dirs.len(), 4);
	}

	#[test]
	fn test_shape_enum() {
		let circle = Shape::Circle { radius: 5.0 };
		let rectangle = Shape::Rectangle {
			width:  10.0,
			height: 20.0,
		};

		match circle {
			Shape::Circle { radius } => assert_eq!(radius, 5.0),
			_ => panic!("Expected Circle variant"),
		}

		match rectangle {
			Shape::Rectangle { width, height } => {
				assert_eq!(width, 10.0);
				assert_eq!(height, 20.0);
			}
			_ => panic!("Expected Rectangle variant"),
		}
	}

	#[test]
	fn test_command_enum() {
		let cmd1 = Command::Execute(42);
		let cmd2 = Command::Delay(std::time::Duration::from_secs(1), 43);
		let cmd3 = Command::Batch(vec![44, 45]);

		match cmd1 {
			Command::Execute(val) => assert_eq!(val, 42),
			_ => panic!("Expected Execute variant"),
		}

		match cmd2 {
			Command::Delay(dur, val) => {
				assert_eq!(dur, std::time::Duration::from_secs(1));
				assert_eq!(val, 43);
			}
			_ => panic!("Expected Delay variant"),
		}

		match cmd3 {
			Command::Batch(vals) => {
				assert_eq!(vals, vec![44, 45]);
			}
			_ => panic!("Expected Batch variant"),
		}
	}
}
