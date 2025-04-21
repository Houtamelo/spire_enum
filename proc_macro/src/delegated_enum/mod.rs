mod conversions;
mod enum_;
mod settings;
mod variants;

use conversions::*;
use enum_::SaneEnum;
use settings::*;
pub use variants::fields::*;
use variants::*;

use super::*;

pub fn run(settings_stream: TokenStream1, enum_stream: TokenStream1) -> Result<TokenStream> {
	let settings = parse_settings(settings_stream)?;
	enum_::run(enum_stream, settings)
}
