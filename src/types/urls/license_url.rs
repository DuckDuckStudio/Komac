use crate::prompts::text::TextPrompt;
use crate::prompts::Prompt;
use crate::types::urls::url::DecodedUrl;
use derive_more::{Display, FromStr};
use serde::{Deserialize, Serialize};

#[derive(Clone, FromStr, Display, Deserialize, Serialize)]
pub struct LicenseUrl(DecodedUrl);

impl Prompt for LicenseUrl {
    const MESSAGE: &'static str = "许可证/文件链接:";
}

impl TextPrompt for LicenseUrl {
    const HELP_MESSAGE: Option<&'static str> = None;
    const PLACEHOLDER: Option<&'static str> = None;
}
