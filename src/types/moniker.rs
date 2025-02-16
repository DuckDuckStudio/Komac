use crate::prompts::text::TextPrompt;
use crate::prompts::Prompt;
use crate::types::tag::Tag;
use derive_more::{Display, FromStr};
use serde::{Deserialize, Serialize};

#[derive(Clone, FromStr, Display, Deserialize, Serialize)]
pub struct Moniker(Tag);

impl Prompt for Moniker {
    const MESSAGE: &'static str = "别名:";
}

impl TextPrompt for Moniker {
    const HELP_MESSAGE: Option<&'static str> = Some("例如: vscode");
    const PLACEHOLDER: Option<&'static str> = None;
}
