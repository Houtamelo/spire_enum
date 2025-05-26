use super::*;

impl CollectIdents for Path {
    fn collect_idents(&self, map: &mut IdentMap) {
        let Self {
            leading_colon: _,
            segments,
        } = self;
        collect!(map, segments);

        if let Some(seg) = segments.last() {
            map.insert_ambiguous(&seg.ident);
        }
    }
}

impl CollectIdents for PathSegment {
    fn collect_idents(&self, map: &mut IdentMap) {
        let Self {
            ident: _,
            arguments,
        } = self;
        collect!(map, arguments);
    }
}

impl CollectIdents for PathArguments {
    fn collect_idents(&self, map: &mut IdentMap) {
        match_collect!(map, self => PathArguments { AngleBracketed, Parenthesized, .. });
    }
}

impl CollectIdents for AssocConst {
    fn collect_idents(&self, map: &mut IdentMap) {
        let Self {
            ident,
            generics,
            eq_token: _,
            value,
        } = self;
        map.insert_constant(ident);
        collect!(map, generics, value);
    }
}

impl CollectIdents for AssocType {
    fn collect_idents(&self, map: &mut IdentMap) {
        let Self {
            ident: _,
            generics,
            eq_token: _,
            ty,
        } = self;
        collect!(map, generics, ty);
    }
}
