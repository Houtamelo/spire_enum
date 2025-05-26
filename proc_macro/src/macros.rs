use super::*;

macro_rules! bail {
	($span:expr => $msg:expr $(,$span_extra:expr => $msg_extra:expr)* $(,)?) => {{
		#[allow(unused_mut)]
		let mut err = Error::new($span.span(), $msg);
		$( err.combine(Error::new($span_extra.span(), $msg_extra)); )*
		return Err(err)
	}};
}

macro_rules! err_expected_only_one {
    ($span1:expr, $span2:expr) => {{
        let err = {
            let mut _temp = Error::new($span1.span(), "expected only one, first occurrence here");
            _temp.combine(Error::new($span2.span(), "second occurrence here"));
            _temp
        };

        return Err(err);
    }};
}

macro_rules! assign_unique_or_panic {
    ($maybe_first:expr, $second:expr) => {
        if let _Some(first) = $maybe_first {
            err_expected_only_one!(first, $second);
        } else {
            $maybe_first = _Some($second);
        }
    };
}

pub(crate) use assign_unique_or_panic;
pub(crate) use bail;
pub(crate) use err_expected_only_one;
