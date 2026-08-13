#![allow(unused)]
use super::*;

#[delegated_enum(impl_conversions, extract_variants(derive(Clone)))]
#[derive(Clone)]
pub enum Message<T> {
    Text(String),
    Binary(Vec<u8>),
    Status(T),
    Nested(Box<Message<T>>),
}

pub trait Var {
    fn var_get(field: &Self) -> Self;
    fn var_set(field: &mut Self, value: Self);
}

impl Var for Text {
    fn var_get(field: &Self) -> Self {
        field.clone()
    }

    fn var_set(field: &mut Self, value: Self) {
        *field = value;
    }
}

impl Var for Binary {
    fn var_get(field: &Self) -> Self {
        field.clone()
    }

    fn var_set(field: &mut Self, value: Self) {
        *field = value;
    }
}

impl<T: Clone> Var for Status<T> {
    fn var_get(field: &Self) -> Self {
        field.clone()
    }

    fn var_set(field: &mut Self, value: Self) {
        *field = value;
    }
}

impl<T: Clone> Var for Nested<T> {
    fn var_get(field: &Self) -> Self {
        field.clone()
    }

    fn var_set(field: &mut Self, value: Self) {
        *field = value;
    }
}

#[delegate_impl]
impl<T: Clone> Var for Message<T> {
    fn var_get(#[receiver] field: &Self) -> Self;

    fn var_set(field: &mut Self, value: Self) {
        *field = value;
    }
}
