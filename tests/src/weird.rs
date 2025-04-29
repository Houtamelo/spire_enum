#![allow(unused)]
use super::*;

#[delegated_enum(extract_variants(derive(Debug, Clone)), impl_conversions)]
#[derive(Clone)]
pub enum Entity<'a, 'c: 'a, T, E: Default, const LEN: usize>
where
	T: Clone,
	for<'b> &'b T: Debug,
	Option<&'c E>: Clone,
{
	SpireWindowMode(Vec<T>),
	WindowSize {
		y: i64,
		x: i32,
	},
	MaxFps(Vec<i32>),
	DialogueTextSpeed(&'a i32),
	Vsync(bool),
	MainVolume(Option<&'c E>),
	MusicVolume(i32, i64, String),
	SfxVolume {
		some_field: i32,
		named_field_hehe: Vec<u32>,
	},
	VoiceVolume {
		lf:  &'a [u32],
		arr: [u32; LEN],
	},
}

#[delegate_impl]
impl<'a, 'c: 'a, T, E: Default + Clone, const LEN: usize> Entity<'a, 'c, T, E, LEN>
where
	T: Clone,
	for<'b> &'b T: Debug,
	Option<&'c E>: Clone,
{
	pub fn clone(&self) -> Self;
}
