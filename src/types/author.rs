use crate::prompts::text::TextPrompt;
use crate::prompts::Prompt;
use nutype::nutype;

#[nutype(
    validate(len_char_min = 2, len_char_max = 256),
    derive(Clone, FromStr, Display, Deserialize, Serialize)
)]
pub struct Author(String);

impl Prompt for Author {
    const MESSAGE: &'static str = "作者:";
}

impl TextPrompt for Author {
    const HELP_MESSAGE: Option<&'static str> = None;
    const PLACEHOLDER: Option<&'static str> = None;
}
