use std::{fmt::Display, ops::BitOr, process};

use bitflags::Flags;
use inquire::{InquireError, MultiSelect, Select, error::InquireResult};
use winget_types::{installer::UpgradeBehavior, shared::value::ValueName};

pub mod list;
pub mod text;

pub trait AllItems {
    type Item: Display;

    fn all() -> impl IntoIterator<Item = Self::Item>;
}

impl AllItems for UpgradeBehavior {
    type Item = Self;

    fn all() -> impl IntoIterator<Item = Self::Item> {
        [
            Self::Item::Install,
            Self::Item::UninstallPrevious,
            Self::Item::Deny,
        ]
    }
}

pub fn radio_prompt<T>() -> InquireResult<T>
where
    T: ValueName + AllItems<Item = T> + Display,
{
    Select::new(
        &format!("{}:", <T as ValueName>::NAME),
        <T as AllItems>::all().into_iter().collect(),
    )
    .prompt()
    .map_err(handle_inquire_error)
}

pub fn check_prompt<T>() -> InquireResult<Option<T>>
where
    T: ValueName + Flags + Display + BitOr<Output = T> + Copy,
{
    MultiSelect::new(
        &format!("{}:", <T as ValueName>::NAME),
        T::all().iter().collect(),
    )
    .prompt()
    .map(|items| {
        if items.is_empty() {
            None
        } else {
            Some(items.iter().fold(T::empty(), |flags, flag| flags | *flag))
        }
    })
    .map_err(handle_inquire_error)
}

/// Inquire captures Ctrl+C and returns an error. This will instead exit normally if the prompt is
/// interrupted.
pub fn handle_inquire_error(error: InquireError) -> InquireError {
    if matches!(
        error,
        InquireError::OperationCanceled | InquireError::OperationInterrupted
    ) {
        process::exit(0);
    }
    error
}
