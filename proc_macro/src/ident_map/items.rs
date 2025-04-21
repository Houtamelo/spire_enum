use super::*;
use crate::delegated_enum::{
	SaneVariantFields,
	SaneVariantFieldsNamed,
	SaneVariantFieldsUnnamed,
	VariantFieldNamed,
	VariantFieldUnnamed,
};

impl CollectIdents for Item {
	fn collect_idents(&self) {
		match_collect!(self => Item {
			Const,
			Enum,
			ExternCrate,
			Fn,
			ForeignMod,
			Impl,
			Macro,
			Mod,
			Static,
			Struct,
			Trait,
			TraitAlias,
			Type,
			Union,
			Use,
			..panic
		});
	}
}

impl CollectIdents for ItemConst {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			vis: _,
			const_token: _,
			ident,
			generics,
			colon_token: _,
			ty,
			eq_token: _,
			expr,
			semi_token: _,
		} = self;
		cache_constant(ident);
		collect!(generics, ty, expr);
	}
}

impl CollectIdents for ItemEnum {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			vis: _,
			enum_token: _,
			ident,
			generics,
			brace_token: _,
			variants,
		} = self;
		cache_ty(ident);
		collect!(generics, variants);
	}
}

impl CollectIdents for Variant {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			ident: _,
			fields,
			discriminant,
		} = self;
		collect!(fields);

		if let Some((_, expr)) = discriminant {
			collect!(expr);
		}
	}
}

impl CollectIdents for ItemForeignMod {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			unsafety: _,
			abi: _,
			brace_token: _,
			items: _, // Maybe someday someone will have the willpower to handle this, I certainly won't
		} = self;
	}
}

impl CollectIdents for ItemImpl {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			defaultness: _,
			unsafety: _,
			impl_token: _,
			generics,
			trait_,
			self_ty,
			brace_token: _,
			items,
		} = self;
		collect!(generics, self_ty, items);

		if let Some((_, trait_path, _)) = trait_ {
			collect!(trait_path);

			if let Some(seg) = trait_path.segments.last() {
				cache_trait(&seg.ident);
			}
		}
	}
}

impl CollectIdents for ImplItem {
	fn collect_idents(&self) {
		match_collect!(self => ImplItem { Const, Fn, Type, Macro, ..panic });
	}
}

impl CollectIdents for ImplItemConst {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			vis: _,
			defaultness: _,
			const_token: _,
			ident,
			generics,
			colon_token: _,
			ty,
			eq_token: _,
			expr,
			semi_token: _,
		} = self;
		cache_constant(ident);
		collect!(generics, ty, expr);
	}
}

impl CollectIdents for ImplItemFn {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			vis: _,
			defaultness: _,
			sig,
			block,
		} = self;
		collect!(sig, block);
	}
}

impl CollectIdents for Signature {
	fn collect_idents(&self) {
		let Self {
			constness: _,
			asyncness: _,
			unsafety: _,
			abi: _,
			fn_token: _,
			ident: _,
			generics,
			paren_token: _,
			inputs,
			variadic: _,
			output,
		} = self;
		collect!(generics, inputs, output);
	}
}

impl CollectIdents for FnArg {
	fn collect_idents(&self) {
		match_collect!(self => FnArg { Receiver, Typed });
	}
}

impl CollectIdents for Receiver {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			reference,
			mutability: _,
			self_token: _,
			colon_token: _,
			ty,
		} = self;
		collect!(ty);

		if let Some((_, lifetime)) = reference {
			collect!(lifetime);
		}
	}
}

impl CollectIdents for ImplItemType {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			vis: _,
			defaultness: _,
			type_token: _,
			ident,
			generics,
			eq_token: _,
			ty,
			semi_token: _,
		} = self;
		cache_ty(ident);
		collect!(generics, ty);
	}
}

impl CollectIdents for ImplItemMacro {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			mac: _,
			semi_token: _,
		} = self;
	}
}

impl CollectIdents for ItemMacro {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			ident: _,
			mac: _,
			semi_token: _,
		} = self;
	}
}

impl CollectIdents for ItemMod {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			vis: _,
			unsafety: _,
			mod_token: _,
			ident: _,
			content,
			semi: _,
		} = self;

		if let Some((_, items)) = content {
			collect!(items);
		}
	}
}

impl CollectIdents for ItemStatic {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			vis: _,
			static_token: _,
			mutability: _,
			ident: _,
			colon_token: _,
			ty,
			eq_token: _,
			expr,
			semi_token: _,
		} = self;
		collect!(ty, expr);
	}
}

impl CollectIdents for ItemStruct {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			vis: _,
			struct_token: _,
			ident,
			generics,
			fields,
			semi_token: _,
		} = self;
		cache_ty(ident);
		collect!(generics, fields);
	}
}

impl CollectIdents for ItemTrait {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			vis: _,
			unsafety: _,
			auto_token: _,
			restriction: _,
			trait_token: _,
			ident,
			generics,
			colon_token: _,
			supertraits,
			brace_token: _,
			items,
		} = self;
		cache_trait(ident);
		collect!(generics, supertraits, items);
	}
}

impl CollectIdents for TraitItem {
	fn collect_idents(&self) {
		match_collect!(self => TraitItem { Const, Fn, Type, Macro, ..panic });
	}
}

impl CollectIdents for TraitItemConst {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			const_token: _,
			ident,
			generics,
			colon_token: _,
			ty,
			default,
			semi_token: _,
		} = self;
		cache_constant(ident);
		collect!(generics, ty);

		if let Some((_, expr)) = default {
			collect!(expr);
		}
	}
}

impl CollectIdents for TraitItemFn {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			sig,
			default,
			semi_token: _,
		} = self;
		collect!(sig, default);
	}
}

impl CollectIdents for TraitItemType {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			type_token: _,
			ident,
			generics,
			colon_token: _,
			bounds,
			default,
			semi_token: _,
		} = self;
		cache_ty(ident);
		collect!(generics, bounds);

		if let Some((_, expr)) = default {
			collect!(expr);
		}
	}
}

impl CollectIdents for TraitItemMacro {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			mac: _,
			semi_token: _,
		} = self;
	}
}

impl CollectIdents for ItemTraitAlias {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			vis: _,
			trait_token: _,
			ident,
			generics,
			eq_token: _,
			bounds,
			semi_token: _,
		} = self;
		cache_trait(ident);
		collect!(generics, bounds);
	}
}

impl CollectIdents for ItemType {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			vis: _,
			type_token: _,
			ident,
			generics,
			eq_token: _,
			ty,
			semi_token: _,
		} = self;
		cache_ty(ident);
		collect!(generics, ty);
	}
}

impl CollectIdents for ItemUnion {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			vis: _,
			union_token: _,
			ident,
			generics,
			fields,
		} = self;
		cache_ty(ident);
		collect!(generics, fields);
	}
}

impl CollectIdents for ItemUse {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			vis: _,
			use_token: _,
			leading_colon: _,
			tree: _, // Maybe someday
			semi_token: _,
		} = self;
	}
}

impl CollectIdents for ItemExternCrate {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			vis: _,
			extern_token: _,
			crate_token: _,
			ident: _,
			rename: _,
			semi_token: _,
		} = self;
	}
}

impl CollectIdents for ItemFn {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			vis: _,
			sig,
			block,
		} = self;
		collect!(sig, block);
	}
}

impl CollectIdents for Fields {
	fn collect_idents(&self) {
		match_collect!(self => Fields { Named, Unnamed, .. });
	}
}

impl CollectIdents for FieldsNamed {
	fn collect_idents(&self) {
		let Self {
			brace_token: _,
			named,
		} = self;
		collect!(named);
	}
}

impl CollectIdents for FieldsUnnamed {
	fn collect_idents(&self) {
		let Self {
			paren_token: _,
			unnamed,
		} = self;
		collect!(unnamed);
	}
}

impl CollectIdents for Field {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			vis: _,
			mutability: _,
			ident: _,
			colon_token: _,
			ty,
		} = self;
		collect!(ty);
	}
}

impl CollectIdents for SaneVariantFields {
	fn collect_idents(&self) {
		match_collect!(self => SaneVariantFields { Named, Unnamed, .. });
	}
}

impl CollectIdents for SaneVariantFieldsNamed {
	fn collect_idents(&self) { iter_collect!(self.fields.iter()) }
}

impl CollectIdents for SaneVariantFieldsUnnamed {
	fn collect_idents(&self) { iter_collect!(self.fields.iter()) }
}

impl<T> CollectIdents for VariantFieldNamed<T> {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			ident: _,
			colon_token: _,
			ty,
		} = self;
		collect!(ty);
	}
}

impl<T> CollectIdents for VariantFieldUnnamed<T> {
	fn collect_idents(&self) {
		let Self { attrs: _, ty } = self;
		collect!(ty);
	}
}
