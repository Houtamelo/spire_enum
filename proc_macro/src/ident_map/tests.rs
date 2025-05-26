use super::*;

#[derive(Debug)]
enum Kind {
    Basic,
    Trait,
    Lifetime,
    Constant,
}

impl IdentMap {
    fn check(mut self, basics: &[&str], traits: &[&str], lifes: &[&str], constants: &[&str]) {
        check_vec(&mut self, basics, Kind::Basic);
        check_vec(&mut self, traits, Kind::Trait);
        check_vec(&mut self, lifes, Kind::Lifetime);
        check_vec(&mut self, constants, Kind::Constant);

        fn check_vec(map: &mut IdentMap, expect: &[&str], kind: Kind) {
            let set = match kind {
                Kind::Basic => &mut map.tys,
                Kind::Trait => &mut map.traits,
                Kind::Lifetime => &mut map.lifetimes,
                Kind::Constant => &mut map.constants,
            };

            let mut missing = Vec::new();

            for &item in expect {
                if !set.remove(item) {
                    missing.push(item);
                }
            }

            if !missing.is_empty() {
                panic!(
                    "Elements({kind:?}) not found: \n\t{}\nRemaining: {map:?}",
                    missing.join("\n\t")
                );
            }

            if !set.is_empty() {
                panic!(
                    "Elements({kind:?}) left: \n\t{}",
                    set.iter().cloned().collect::<Vec<_>>().join("\n\t")
                );
            }
        }
    }
}

macro_rules! test_case {
	($input: path => $($basics:literal)* , $($traits:literal)* , $($lifes:literal)*, $($constants:literal)*) => {
	    let input: Type = parse_quote!($input);
		let idents = IdentMap::new(&input);
		idents.check(&[$($basics),*], &[$($traits),*], &[$($lifes),*], &[$($constants),*]);
    };

	($input: ty => $($basics:literal)* , $($traits:literal)* , $($lifes:literal)*, $($constants:literal)*) => {
	    let input: Type = parse_quote!($input);
		let idents = IdentMap::new(&input);
		idents.check(&[$($basics),*], &[$($traits),*], &[$($lifes),*], &[$($constants),*]);
    };
}

macro_rules! test_case_tt {
	([$($input: tt)*] => $($basics:literal)* , $($traits:literal)* , $($lifes:literal)*, $($constants:literal)*) => {
	    let input: Type = parse_quote!($($input)*);
		let idents = IdentMap::new(&input);
		idents.check(&[$($basics),*], &[$($traits),*], &[$($lifes),*], &[$($constants),*]);
    };

	($Ty: ty : [$($input: tt)*] => $($basics:literal)* , $($traits:literal)* , $($lifes:literal)*, $($constants:literal)*) => {
	    let input: $Ty = parse_quote!($($input)*);
		let idents = IdentMap::new(&input);
		idents.check(&[$($basics),*], &[$($traits),*], &[$($lifes),*], &[$($constants),*]);
    };
}

// i32
// String
// bool
// Option<T>
// Result<T, E>
#[test]
fn plain_types() {
    test_case! { i32 => "i32",,, }
    test_case! { String => "String",,, }
    test_case! { bool => "bool",,, }
    test_case! { Option<T> => "Option" "T",,, }
    test_case! { Result<T, E> => "Result" "T" "E",,, }
}

// &'a str
// &'static [u8]
// &'a mut Vec<T>
#[test]
fn references() {
    test_case! { &String => "String",,, }
    test_case! { &'a str => "str",,"a", }
    test_case! { &'static [u8] => "u8",,"static", }
    test_case! { &'a mut Vec<T> => "Vec" "T",,"a", }
}

// Vec<T>
// HashMap<K, V>
// BTreeMap<K, V>
// HashSet<T>
#[test]
fn collections() {
    test_case! { Vec<T> => "Vec" "T",,, }
    test_case! { HashMap<K, V> => "HashMap" "K" "V",,, }
    test_case! { BTreeMap<K, V> => "BTreeMap" "K" "V",,, }
    test_case! { HashSet<T> => "HashSet" "T",,, }
}

// [T; N]
// [T]
// &[T]
// &mut [T; 5]
#[test]
fn array_slices() {
    test_case! { [T; N] => "T",,, }
    test_case! { [T] => "T",,, }
    test_case! { &[T] => "T",,, }
    test_case! { &mut [T; 5] => "T",,, }
    test_case! { &'a mut [T; LEN] => "T",,"a", }
}

// (T, U)
// (A, B, C, D)
// ()
#[test]
fn tuples() {
    test_case! { (T, U) => "T" "U",,, }
    test_case! { (A, B, C, D) => "A" "B" "C" "D",,, }
    test_case! { (&A, &'c B, &'t C, &mut D) => "A" "B" "C" "D",,"c""t", }
    test_case! { () => ,,, }
}

// fn(T) -> U
// fn(i32, String) -> bool
// Fn(T) -> U
// FnMut() -> ()
#[test]
fn functions() {
    test_case! { [fn(T) -> U] => "T" "U",,, }
    test_case! { [fn(i32, String) -> bool] => "i32" "String" "bool",,, }
    test_case! { [fn(i32, String) -> impl Iterator<Item = SomeType>] => "i32" "String" "SomeType","Iterator",, }
    test_case! { [fn(&'b i32, &mut String) -> impl Iterator<'a, Item = SomeType>] => "i32" "String" "SomeType","Iterator", "a""b", }
    test_case! { [dyn Fn(T) -> U] => "T" "U","Fn",, }
    test_case! { [dyn Fn(T, &U, &'c F) -> dyn Clone] => "T" "U" "F","Fn" "Clone", "c", }
    test_case! { [dyn FnMut() -> ()] => ,"FnMut",, }
}

// Box<dyn Trait>
// &dyn Iterator<Item = T>
// &mut dyn Write
#[test]
fn trait_objs() {
    test_case! { Box<dyn Trait> => "Box","Trait",, }
    test_case! { &dyn Iterator<Item = T> => "T","Iterator",, }
    test_case! { &'a dyn Iterator<Item = T<U, 'd>> => "T""U","Iterator","a""d", }
    test_case! { &mut dyn Write => ,"Write",, }
    test_case! { &'static dyn Write => ,"Write", "static", }
}

// T: Clone + Send
// T: Into<String> + 'static
// T: AsRef<[u8]> + ?Sized
#[test]
fn generics_with_bounds() {
    test_case_tt! { TypeParam: [T: Clone + Send] => "T", "Clone""Send",, }
    test_case_tt! { TypeParam: [T: Into<String> + 'static] => "T""String", "Into","static", }
    test_case_tt! { TypeParam: [T: AsRef<[u8]> + ?Sized] => "T" "u8", "AsRef" "Sized",, }
    test_case_tt! { TypeParam: [T: AsRef<&[u8]> + ?Sized + Deserialize<'a>] => "T" "u8", "AsRef" "Sized" "Deserialize","a", }
}

// 'a: 'b
// T: 'a
// for<'a> &'a T
#[test]
fn lifetime_bounds() {
    test_case_tt! { LifetimeParam: ['a: 'b] => ,,"a""b", }
    test_case_tt! { TypeParam: [T: 'static] => "T",,"static", }
    test_case_tt! { WherePredicate: [for<'a> &'a mut T: Deserialize<'a>] => "T","Deserialize","a", }
}

// Result<Vec<Option<T>>, Box<dyn Error>>
// HashMap<String, Vec<(i32, bool)>>
// Box<dyn Fn(&'a str) -> Result<T, E>>
#[test]
fn complex_nested_types() {
    test_case! { Result<Vec<Option<T>>, Box<dyn Error>> => "Result" "Vec" "Option" "T" "Box", "Error",, }
    test_case! { HashMap<String, Vec<(i32, bool)>> => "HashMap" "String" "Vec" "i32" "bool",,, }
    test_case! { Box<dyn Fn(&'a str) -> Result<T, E>> => "Box" "str" "Result" "T" "E", "Fn", "a", }
}

// impl Iterator<Item = T>
// impl Trait + 'static
// dyn for<'a> Fn(&'a str) -> &'a str
#[test]
fn advanced_generics() {
    test_case_tt! { [impl Iterator<Item = T>] => "T","Iterator",, }
    test_case_tt! { [impl Trait + 'static] => ,"Trait","static",}
    test_case_tt! { [dyn for<'a> Fn(&'a str) -> &'a str] => "str","Fn","a", }
}

// <T as Iterator>::Item
// <Self as AsRef<[u8]>>::Target
#[test]
fn associated_types() {
    test_case! { <T as Iterator>::Item => "T" "Item", "Iterator",, }
    test_case! { <Self as AsRef<[u8]>>::Target => "Self" "u8" "Target", "AsRef",,  }
    test_case! { <Self as AsRef<[u8]>>::Target::<'a> => "Self" "u8" "Target", "AsRef", "a", }
}

// fn(T) -> impl Future<Output = Result<Vec<U>, Box<dyn Error + Send + 'static>>>
// for<'a> fn(&'a [T]) -> impl Iterator<Item = &'a U> + 'a
// Box<dyn for<'r> FnMut(&'r str) -> Option<&'r [u8]>>
// Rc<RefCell<HashMap<String, Box<dyn Any>>>>
// Pin<Box<dyn Future<Output = Result<T, E>> + Send + 'static>>
// Arc<Mutex<Vec<Box<dyn Fn() -> Result<(), Box<dyn Error>> + Send + 'static>>>>
// Either<Box<dyn Iterator<Item = T>>, impl IntoIterator<Item = T>>
// PhantomData<fn() -> T>
// fn<'a, T: Trait<'a> + 'a>(&'a mut [T]) -> impl Iterator<Item = &'a T::Output> + 'a
// <T as SomeTrait<'a, U>>::AssocType<V, W>
#[test]
fn extra_complex() {
    test_case_tt! {
        [fn(T) -> impl Future<Output = Result<Vec<U>, Box<dyn Error + Send + 'static>>>] =>
        "Result" "Vec" "U" "Box" "T", "Future" "Error" "Send", "static",
    }

    test_case_tt! {
        [for<'a, 'b> fn(&'a [T]) -> (dyn Iterator<Item = &'b U> + 'a)] =>
        "T" "U","Iterator","a""b",
    }

    test_case_tt! {
        [Box<dyn for<'r> FnMut(&'r str) -> Option<&'r [u8]>>] =>
        "Box" "str" "Option" "u8", "FnMut", "r",
    }

    test_case! {
        Rc<RefCell<HashMap<String, Box<dyn Any>>>> =>
        "Rc" "RefCell" "HashMap" "String" "Box", "Any",,
    }

    test_case! {
        Pin<Box<dyn Future<Output = Result<T, E>> + Send + 'static>> =>
        "Pin" "Box" "Result" "T" "E", "Future" "Send", "static",
    }

    test_case! {
        Arc<Mutex<Vec<Box<dyn Fn() -> Result<(), Box<dyn Error>> + Send + 'static>>>> =>
        "Arc" "Mutex" "Vec" "Box" "Result", "Fn" "Error" "Send", "static",
    }

    test_case! {
        Either<Box<dyn Iterator<Item = T>>, impl IntoIterator<Item = T>> =>
        "Either" "Box" "T", "Iterator" "IntoIterator",,
    }

    test_case! {
        PhantomData<fn() -> T> =>
        "PhantomData" "T",,,
    }

    test_case_tt! {
        [fn(&'a mut [T]) -> (dyn Iterator<Item = &'a T::Output> + 'a)] =>
        "T" "Output", "Iterator", "a",
    }

    test_case! {
        <T as SomeTrait<'a, U>>::AssocType<V, W> =>
        "T" "U" "AssocType" "V" "W", "SomeTrait", "a",
    }
}

#[test]
fn constants() {
    test_case_tt! { Generics: [<const LEN: usize>] => "usize",,,"LEN" }
    test_case_tt! { Generics: [<T, const LEN: usize>] => "usize""T",,,"LEN" }
    test_case_tt! { Generics: [<'a, F, const LEN: usize>] => "usize""F",,"a","LEN" }
    test_case_tt! { Generics: [<'b, const LEN: usize, const SIZE: isize, T>] => "usize""isize""T",,"b","LEN""SIZE" }
}

// HashMap<&'a str, Vec<Rc<RefCell<Option<Box<dyn Trait<'b> + 'static>>>>>>
// for<'a, 'b> fn(&'a Foo<'b>) -> &'b Bar<'a>
// T: for<'r> Fn(&'r [u8]) -> Result<&'r str, &'r Error>
#[test]
fn nested_generics_with_lifetimes() {
    test_case! {
        HashMap<&'a str, Vec<Rc<RefCell<Option<Box<dyn Trait<'b> + 'static>>>>>> =>
        "HashMap" "str" "Vec" "Rc" "RefCell" "Option" "Box", "Trait", "a" "b" "static",
    }

    test_case! {
        [for<'a, 'b> fn(&'a Foo<'b>) -> &'b Bar<'a>] =>
        "Foo" "Bar",, "a" "b",
    }

    test_case_tt! {
        WherePredicate: [T: for<'r> Fn(&'r [u8]) -> Result<&'r str, &'r Error>] =>
        "T" "u8" "str" "Result" "Error", "Fn", "r",
    }
}

#[test]
fn test_1() {
    //T: Into<String>, 'a: 'b, const LEN: usize.
    let input: GenericParam =
        parse_quote!(T: Into<Vec<([N; LEN], &'b mut [dyn Clone + Iterator<Item = *const T>])>>);
    let idents = IdentMap::new(&input);

    idents.check(&["T", "Vec", "N"], &["Into", "Iterator", "Clone"], &["b"], &[]);
}

#[test]
fn test_2() {
    let input: GenericParam = parse_quote!('a: 'b);
    let idents = IdentMap::new(&input);

    idents.check(&[], &[], &["a", "b"], &[]);
}

#[test]
fn test_3() {
    let input: BoundLifetimes = parse_quote!(for<'a, 'b>);
    let idents = IdentMap::new(&input);

    idents.check(&[], &[], &["a", "b"], &[]);
}

#[test]
fn test_4() {
    let input: TraitBound = parse_quote!(for<'a, 'b> some_module::T<'a>);
    let idents = IdentMap::new(&input);

    idents.check(&[], &["T"], &["a", "b"], &[]);
}

#[test]
fn test_5() {
    let input: Type = parse_quote!((T<N, 'a>));
    let idents = IdentMap::new(&input);

    idents.check(&["T", "N"], &[], &["a"], &[]);
}

#[test]
fn test_6() {
    let input: Type = parse_quote!(fn(usize, T, &'c N, [E; LEN]) -> impl Deref<Target = A<'b>>);
    let idents = IdentMap::new(&input);

    idents.check(&["usize", "T", "N", "E", "A"], &["Deref"], &["c", "b"], &[]);
}

#[test]
fn test_7() {
    let input: Type = parse_quote!(
        <Self as SomeTrait>::SomeAssocType<
            fn(usize, T, &'c N, [E; LEN]) -> impl Deref<Target = A<'b>>,
        >
    );
    let idents = IdentMap::new(&input);

    idents.check(
        &["Self", "SomeAssocType", "usize", "T", "N", "E", "A"],
        &["SomeTrait", "Deref"],
        &["c", "b"],
        &[],
    );
}
