#![allow(unused)]
use std::marker::PhantomData;

use super::*;

#[derive(Clone, Debug)]
pub struct State<T: ?Sized>(T);

#[derive(Clone, Debug)]
pub struct Dummy<'a, 'b, S, T>(PhantomData<(&'a S, &'b T)>)
where
	'b: 'a,
	T: Sized;

#[delegated_enum]
#[derive(Clone, Debug)]
enum StateEnum<'a, 'b, S, T>
where
	'b: 'a,
	T: Sized,
{
	Int(State<i32>),
	Sneaky(i32, #[delegator] State<u32>),
	Empty {
		dummy: Dummy<'a, 'b, S, T>,
	},
	DumDum {
		pardon_me: i32,
		#[delegator]
		dummy: Dummy<'a, 'b, S, T>,
	},
	DumDumSquared {
		#[delegator]
		dummy: Dummy<'a, 'b, S, T>,
		pardon_me: i32,
	},
	#[delegate_via(|p, d| d)]
	DumDumTriple {
		pardon_me: i32,
		dummy: Dummy<'a, 'b, S, T>,
	},
}

#[delegate_impl]
impl<'a, 'b, S, T> StateEnum<'a, 'b, S, T> {
	fn do_nothing(&self);
	fn do_something(&self, arg1: i32);
}

impl<T> State<T> {
	fn do_nothing(&self) {}
	fn do_something(&self, _arg1: i32) {}
}

impl<'a, 'b, S, T> Dummy<'a, 'b, S, T> {
	fn do_nothing(&self) {}
	fn do_something(&self, _arg1: i32) {}
}

fn test<S: Clone, T: Clone>(input: StateEnum<S, T>) {
	let s = input.clone();
	s.do_nothing();
}
