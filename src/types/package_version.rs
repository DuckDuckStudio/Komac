use crate::prompts::text::TextPrompt;
use crate::prompts::Prompt;
use crate::types::version::Version;
use crate::types::DISALLOWED_CHARACTERS;
use derive_more::{Deref, Display, Into};
use serde_with::{DeserializeFromStr, SerializeDisplay};
use std::str::FromStr;
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum PackageVersionError {
    #[error("软件包版本不得包含任何控制字符")]
    ContainsControlChars,
    #[error(
        "软件包版本不得包含以下任何字符: {:?}",
        DISALLOWED_CHARACTERS
    )]
    DisallowedCharacters,
    #[error(
        "软件包版本不得长于 {} 字符",
        PackageVersion::MAX_LENGTH
    )]
    TooLong,
}

#[derive(
    Clone,
    Debug,
    Default,
    Deref,
    Display,
    Into,
    Hash,
    Eq,
    Ord,
    PartialEq,
    PartialOrd,
    SerializeDisplay,
    DeserializeFromStr,
)]
#[into(ref)]
pub struct PackageVersion(Version);

impl PackageVersion {
    const MAX_LENGTH: usize = 1 << 7;
}

impl FromStr for PackageVersion {
    type Err = PackageVersionError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        if input.contains(DISALLOWED_CHARACTERS) {
            return Err(PackageVersionError::DisallowedCharacters);
        } else if input.contains(char::is_control) {
            return Err(PackageVersionError::ContainsControlChars);
        } else if input.chars().count() > Self::MAX_LENGTH {
            return Err(PackageVersionError::TooLong);
        }

        Ok(Self(Version::new(input)))
    }
}

impl Prompt for PackageVersion {
    const MESSAGE: &'static str = "软件包版本:";
}

impl TextPrompt for PackageVersion {
    const HELP_MESSAGE: Option<&'static str> = Some("例如: 1.2.3 (应与控制面板或设置中的版本号相匹配)"); // 温馨提示，欸嘿
    const PLACEHOLDER: Option<&'static str> = None;
}

#[cfg(test)]
mod tests {
    use crate::types::package_version::{PackageVersion, PackageVersionError};
    use crate::types::DISALLOWED_CHARACTERS;
    use const_format::str_repeat;
    use std::str::FromStr;

    #[test]
    fn package_version_disallowed_characters() {
        for char in DISALLOWED_CHARACTERS {
            let version = format!("1.2{char}3");

            assert_eq!(
                PackageVersion::from_str(&version).err(),
                Some(PackageVersionError::DisallowedCharacters)
            )
        }
    }

    #[test]
    fn package_version_contains_control_chars() {
        assert_eq!(
            PackageVersion::from_str("1.2\03").err(),
            Some(PackageVersionError::ContainsControlChars)
        );
    }

    #[test]
    fn unicode_package_version_max_length() {
        const VERSION: &str = str_repeat!("🦀", PackageVersion::MAX_LENGTH);

        // Ensure that it's character length that's being checked and not byte or UTF-16 length
        assert!(VERSION.len() > PackageVersion::MAX_LENGTH);
        assert!(VERSION.encode_utf16().count() > PackageVersion::MAX_LENGTH);
        assert_eq!(VERSION.chars().count(), PackageVersion::MAX_LENGTH);
        assert!(VERSION.parse::<PackageVersion>().is_ok());
    }

    #[test]
    fn package_version_too_long() {
        const VERSION: &str = str_repeat!("🦀", PackageVersion::MAX_LENGTH + 1);

        assert_eq!(
            VERSION.parse::<PackageVersion>().err(),
            Some(PackageVersionError::TooLong)
        );
    }
}
