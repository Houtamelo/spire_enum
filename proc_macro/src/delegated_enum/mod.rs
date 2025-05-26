mod conversions;
mod enum_;
mod settings;
mod variant;
mod variant_fields;

use conversions::*;
use enum_::SaneEnum;
use settings::*;
use variant::*;
use variant_fields::*;

use super::*;

pub fn run(settings_stream: TokenStream1, enum_stream: TokenStream1) -> Result<TokenStream> {
    let settings = parse_settings(settings_stream.into())?;
    enum_::run(enum_stream, settings)
}
