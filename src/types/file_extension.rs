use crate::prompts::list::ListPrompt;
use crate::prompts::Prompt;
use nutype::nutype;

#[nutype(
    validate(not_empty, len_char_max = 512),
    derive(
        Clone,
        Debug,
        Deserialize,
        Display,
        Eq,
        FromStr,
        Ord,
        PartialEq,
        PartialOrd,
        Serialize,
        Hash
    )
)]
pub struct FileExtension(String);

impl Prompt for FileExtension {
    const MESSAGE: &'static str = "文件类型:";
}

impl ListPrompt for FileExtension {
    const HELP_MESSAGE: &'static str = "软件包可以支持的文件类型的列表";
    const MAX_ITEMS: u16 = 512;
}
