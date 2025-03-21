use anstream::println;
use clap::Parser;
use color_eyre::eyre::Result;
use owo_colors::OwoColorize;

use crate::{credential::get_komac_credential, prompts::text::confirm_prompt};

/// Remove the stored token
#[derive(Parser)]
#[clap(visible_alias = "delete")]
pub struct RemoveToken {
    /// 跳过删除令牌的确认提示
    #[arg(short = 'y', long = "yes")]
    skip_prompt: bool,
}

impl RemoveToken {
    pub fn run(self) -> Result<()> {
        let credential = get_komac_credential()?;

        if matches!(
            credential.get_password().err(),
            Some(keyring::Error::NoEntry)
        ) {
            println!("当前没有存储在平台安全存储中的令牌");
        }

        let confirm = if self.skip_prompt {
            true
        } else {
            confirm_prompt("是否删除当前存储的令牌?")?
        };

        if confirm {
            credential.delete_credential()?;
            println!(
                "{} 已从平台的安全存储中删除存储的令牌",
                "成功".green()
            );
        } else {
            println!("{}", "没有删除任何令牌".cyan());
        }

        Ok(())
    }
}
