use super::*;

impl CollectIdents for Expr {
	fn collect_idents(&self) {
		match_collect!(self => Expr {
			Array,
			Assign,
			Async,
			Await,
			Binary,
			Block,
			Break,
			Call,
			Cast,
			Closure,
			Const,
			Continue,
			Field,
			ForLoop,
			Group,
			If,
			Index,
			Infer,
			Let,
			Lit,
			Loop,
			Macro,
			Match,
			MethodCall,
			Paren,
			Path,
			Range,
			RawAddr,
			Reference,
			Repeat,
			Return,
			Struct,
			Try,
			TryBlock,
			Tuple,
			Unary,
			Unsafe,
			While,
			Yield,
			..panic
		});
	}
}

impl CollectIdents for ExprArray {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			bracket_token: _,
			elems,
		} = self;
		collect!(elems);
	}
}

impl CollectIdents for ExprAssign {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			left,
			eq_token: _,
			right,
		} = self;
		collect!(left, right);
	}
}

impl CollectIdents for ExprAsync {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			async_token: _,
			capture: _,
			block,
		} = self;
		collect!(block);
	}
}

impl CollectIdents for ExprAwait {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			base,
			dot_token: _,
			await_token: _,
		} = self;
		collect!(base);
	}
}

impl CollectIdents for ExprBinary {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			left,
			op: _,
			right,
		} = self;
		collect!(left, right);
	}
}

impl CollectIdents for ExprBlock {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			label: _,
			block,
		} = self;
		collect!(block);
	}
}

impl CollectIdents for ExprBreak {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			break_token: _,
			label: _,
			expr,
		} = self;
		collect!(expr);
	}
}

impl CollectIdents for ExprCall {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			func,
			paren_token: _,
			args,
		} = self;
		collect!(func, args);
	}
}

impl CollectIdents for ExprCast {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			expr,
			as_token: _,
			ty,
		} = self;
		collect!(expr, ty);
	}
}

impl CollectIdents for ExprClosure {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			lifetimes,
			constness: _,
			movability: _,
			asyncness: _,
			capture: _,
			or1_token: _,
			inputs,
			or2_token: _,
			output,
			body,
		} = self;
		collect!(lifetimes, output, body, inputs);
	}
}

impl CollectIdents for ExprConst {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			const_token: _,
			block,
		} = self;
		collect!(block);
	}
}

impl CollectIdents for ExprContinue {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			continue_token: _,
			label: _,
		} = self;
	}
}

impl CollectIdents for ExprField {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			base,
			dot_token: _,
			member,
		} = self;
		collect!(base, member);
	}
}

impl CollectIdents for ExprForLoop {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			label: _,
			for_token: _,
			pat,
			in_token: _,
			expr,
			body,
		} = self;
		collect!(pat, expr, body);
	}
}

impl CollectIdents for ExprGroup {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			group_token: _,
			expr,
		} = self;
		collect!(expr);
	}
}

impl CollectIdents for ExprIf {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			if_token: _,
			cond,
			then_branch,
			else_branch,
		} = self;
		collect!(cond, then_branch);

		if let Some((_, block)) = else_branch {
			collect!(block);
		}
	}
}

impl CollectIdents for ExprIndex {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			expr,
			bracket_token: _,
			index,
		} = self;
		collect!(expr, index);
	}
}

impl CollectIdents for ExprInfer {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			underscore_token: _,
		} = self;
	}
}

impl CollectIdents for ExprLet {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			let_token: _,
			pat,
			eq_token: _,
			expr,
		} = self;
		collect!(pat, expr);
	}
}

impl CollectIdents for ExprLit {
	fn collect_idents(&self) { let Self { attrs: _, lit: _ } = self; }
}

impl CollectIdents for ExprLoop {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			label: _,
			loop_token: _,
			body,
		} = self;
		collect!(body);
	}
}

impl CollectIdents for ExprMacro {
	fn collect_idents(&self) { let Self { attrs: _, mac: _ } = self; }
}

impl CollectIdents for ExprMatch {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			match_token: _,
			expr,
			brace_token: _,
			arms,
		} = self;
		collect!(expr, arms);
	}
}

impl CollectIdents for ExprMethodCall {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			receiver,
			dot_token: _,
			method: _,
			turbofish,
			paren_token: _,
			args,
		} = self;
		collect!(receiver, turbofish, args);
	}
}

impl CollectIdents for ExprParen {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			paren_token: _,
			expr,
		} = self;
		collect!(expr);
	}
}

impl CollectIdents for ExprPath {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			qself,
			path,
		} = self;
		collect!(qself, path);

		if let Some(QSelf {
			position: pos @ 1..,
			as_token: Some(..),
			..
		}) = &qself
		{
			let trait_segment = &path.segments[pos - 1];
			cache_trait(&trait_segment.ident);
		}
	}
}

impl CollectIdents for ExprRange {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			start,
			limits: _,
			end,
		} = self;
		collect!(start, end);
	}
}

impl CollectIdents for ExprRawAddr {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			and_token: _,
			raw: _,
			mutability: _,
			expr,
		} = self;
		collect!(expr);
	}
}

impl CollectIdents for ExprReference {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			and_token: _,
			mutability: _,
			expr,
		} = self;
		collect!(expr);
	}
}

impl CollectIdents for ExprRepeat {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			bracket_token: _,
			expr,
			semi_token: _,
			len,
		} = self;
		collect!(expr, len);
	}
}

impl CollectIdents for ExprReturn {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			return_token: _,
			expr,
		} = self;
		collect!(expr);
	}
}

impl CollectIdents for ExprStruct {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			qself,
			path,
			brace_token: _,
			fields,
			dot2_token: _,
			rest,
		} = self;
		collect!(qself, path, rest, fields);

		if let Some(QSelf {
			position: pos @ 1..,
			as_token: Some(..),
			..
		}) = &qself
		{
			let trait_segment = &path.segments[pos - 1];
			cache_trait(&trait_segment.ident);
		}

		if let Some(seg) = path.segments.last() {
			cache_ty(&seg.ident);
		}
	}
}

impl CollectIdents for ExprTry {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			expr,
			question_token: _,
		} = self;
		collect!(expr);
	}
}

impl CollectIdents for ExprTuple {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			paren_token: _,
			elems,
		} = self;
		collect!(elems);
	}
}

impl CollectIdents for ExprUnary {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			op: _,
			expr,
		} = self;
		collect!(expr);
	}
}

impl CollectIdents for ExprUnsafe {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			unsafe_token: _,
			block,
		} = self;
		collect!(block);
	}
}

impl CollectIdents for ExprWhile {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			label: _,
			while_token: _,
			cond,
			body,
		} = self;
		collect!(cond, body);
	}
}

impl CollectIdents for ExprYield {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			yield_token: _,
			expr,
		} = self;
		collect!(expr);
	}
}

impl CollectIdents for ExprTryBlock {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			try_token: _,
			block,
		} = self;
		collect!(block);
	}
}

impl CollectIdents for Block {
	fn collect_idents(&self) {
		let Self {
			brace_token: _,
			stmts,
		} = self;
		collect!(stmts);
	}
}

impl CollectIdents for Stmt {
	fn collect_idents(&self) {
		match self {
			Stmt::Local(local) => collect!(local),
			Stmt::Item(item) => collect!(item),
			Stmt::Expr(expr, _) => collect!(expr),
			Stmt::Macro(mac) => collect!(mac),
		}
	}
}

impl CollectIdents for Local {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			let_token: _,
			pat,
			init,
			semi_token: _,
		} = self;
		collect!(pat, init);
	}
}

impl CollectIdents for LocalInit {
	fn collect_idents(&self) {
		let Self {
			eq_token: _,
			expr,
			diverge,
		} = self;
		collect!(expr);

		if let Some((_, block)) = diverge {
			collect!(block);
		}
	}
}

impl CollectIdents for StmtMacro {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			mac: _,
			semi_token: _,
		} = self;
	}
}

impl CollectIdents for Arm {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			pat,
			guard,
			fat_arrow_token: _,
			body,
			comma: _,
		} = self;
		collect!(pat, body);

		if let Some((_, expr)) = guard {
			collect!(expr);
		}
	}
}

impl CollectIdents for Member {
	fn collect_idents(&self) {} // nothing to do
}

impl CollectIdents for FieldValue {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			member,
			colon_token: _,
			expr,
		} = self;
		collect!(member, expr);
	}
}
