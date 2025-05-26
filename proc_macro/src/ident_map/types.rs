use super::*;

impl CollectIdents for TypeArray {
    fn collect_idents(&self, map: &mut IdentMap) {
        let Self {
            bracket_token: _,
            elem,
            semi_token: _,
            len,
        } = self;
        collect!(map, elem, len);
    }
}

impl CollectIdents for TypeSlice {
    fn collect_idents(&self, map: &mut IdentMap) {
        let Self {
            bracket_token: _,
            elem,
        } = self;
        collect!(map, elem);
    }
}

impl CollectIdents for TypeGroup {
    fn collect_idents(&self, map: &mut IdentMap) {
        let Self {
            group_token: _,
            elem,
        } = self;
        collect!(map, elem);
    }
}

impl CollectIdents for TypeParen {
    fn collect_idents(&self, map: &mut IdentMap) {
        let Self {
            paren_token: _,
            elem,
        } = self;
        collect!(map, elem);
    }
}

impl CollectIdents for TypePtr {
    fn collect_idents(&self, map: &mut IdentMap) {
        let Self {
            star_token: _,
            const_token: _,
            mutability: _,
            elem,
        } = self;
        collect!(map, elem);
    }
}

impl CollectIdents for TypeBareFn {
    fn collect_idents(&self, map: &mut IdentMap) {
        let Self {
            lifetimes,
            unsafety: _,
            abi: _,
            fn_token: _,
            paren_token: _,
            inputs,
            variadic: _,
            output,
        } = self;
        collect!(map, lifetimes, inputs, output);
    }
}

impl CollectIdents for BareFnArg {
    fn collect_idents(&self, map: &mut IdentMap) {
        let Self {
            attrs: _,
            name: _,
            ty,
        } = self;
        collect!(map, ty);
    }
}

impl CollectIdents for TypeImplTrait {
    fn collect_idents(&self, map: &mut IdentMap) {
        let Self {
            impl_token: _,
            bounds,
        } = self;
        collect!(map, bounds);
    }
}

impl CollectIdents for TypeTraitObject {
    fn collect_idents(&self, map: &mut IdentMap) {
        let Self {
            dyn_token: _,
            bounds,
        } = self;
        collect!(map, bounds);
    }
}

impl CollectIdents for TypeTuple {
    fn collect_idents(&self, map: &mut IdentMap) {
        let Self {
            paren_token: _,
            elems,
        } = self;
        collect!(map, elems);
    }
}

impl CollectIdents for TypeReference {
    fn collect_idents(&self, map: &mut IdentMap) {
        let Self {
            and_token: _,
            lifetime,
            mutability: _,
            elem,
        } = self;
        collect!(map, lifetime, elem);
    }
}

impl CollectIdents for TypeInfer {
    fn collect_idents(&self, _map: &mut IdentMap) {
        let Self {
            underscore_token: _,
        } = self;
    }
}

impl CollectIdents for TypeMacro {
    fn collect_idents(&self, _map: &mut IdentMap) { let Self { mac: _ } = self; }
}

impl CollectIdents for TypeNever {
    fn collect_idents(&self, _map: &mut IdentMap) { let Self { bang_token: _ } = self; }
}

impl CollectIdents for Type {
    fn collect_idents(&self, map: &mut IdentMap) {
        match_collect!(map, self => Type {
            Array,
            Slice,
            Group,
            Paren,
            Ptr,
            Path,
            BareFn,
            ImplTrait,
            Reference,
            TraitObject,
            Tuple,
            Infer,
            Macro,
            Never,
            ..panic
        })
    }
}

impl CollectIdents for QSelf {
    fn collect_idents(&self, map: &mut IdentMap) {
        let Self {
            lt_token: _,
            ty,
            position: _,
            as_token: _,
            gt_token: _,
        } = self;
        collect!(map, ty);
    }
}

impl CollectIdents for TypePath {
    fn collect_idents(&self, map: &mut IdentMap) {
        let Self { qself, path } = self;
        collect!(map, qself, path);

        if let Some(QSelf {
            position: pos @ 1..,
            as_token: Some(..),
            ..
        }) = &qself
        {
            let trait_segment = &path.segments[pos - 1];
            map.insert_trait(&trait_segment.ident);
        }

        if let Some(seg) = path.segments.last() {
            map.insert_ty(&seg.ident);
        }
    }
}

impl CollectIdents for ReturnType {
    fn collect_idents(&self, map: &mut IdentMap) {
        if let ReturnType::Type(_, ty) = self {
            collect!(map, ty);
        }
    }
}
