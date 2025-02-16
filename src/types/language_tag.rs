use crate::prompts::text::TextPrompt;
use crate::prompts::Prompt;
use derive_more::{AsRef, Deref, Display, FromStr};
use icu_locid::{langid, LanguageIdentifier};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[derive(
    AsRef, Clone, Debug, Deref, Display, Deserialize, Serialize, FromStr, Eq, PartialEq, Hash,
)]
pub struct LanguageTag(LanguageIdentifier);

impl Default for LanguageTag {
    fn default() -> Self { // 默认
        Self(langid!("zh-CN"))
    }
}

impl Ord for LanguageTag {
    fn cmp(&self, other: &Self) -> Ordering {
        other.0.total_cmp(&self.0)
    }
}

impl PartialOrd for LanguageTag {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Prompt for LanguageTag {
    const MESSAGE: &'static str = "软件包区域:";
}

impl TextPrompt for LanguageTag {
    const HELP_MESSAGE: Option<&'static str> = Some("例如: zh-CN (简体中文)");
    const PLACEHOLDER: Option<&'static str> = None;
}
