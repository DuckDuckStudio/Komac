use crate::prompts::list::ListPrompt;
use crate::prompts::Prompt;
use nutype::nutype;

#[nutype(
    validate(len_char_min = 1, len_char_max = 40),
    derive(
        Clone,
        FromStr,
        Display,
        Deserialize,
        Serialize,
        Eq,
        PartialEq,
        Ord,
        PartialOrd
    )
)]
pub struct Tag(String);

impl Prompt for Tag {
    const MESSAGE: &'static str = "标记:";
}

impl ListPrompt for Tag {
    const HELP_MESSAGE: &'static str = "例如: zip, c++, 图片, OBS, 音乐";
    const MAX_ITEMS: u16 = 16;
}
