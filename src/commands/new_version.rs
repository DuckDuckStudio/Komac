use std::{
    collections::BTreeSet,
    mem,
    num::{NonZeroU8, NonZeroU32},
};

use anstream::println;
use camino::Utf8PathBuf;
use clap::Parser;
use color_eyre::eyre::Result;
use indicatif::ProgressBar;
use inquire::CustomType;
use ordinal_trait::Ordinal;
use owo_colors::OwoColorize;
use reqwest::Client;
use winget_types::{
    installer::{
        Command, FileExtension, InstallModes, InstallerManifest, InstallerSuccessCode,
        InstallerType, Protocol, UpgradeBehavior,
        switches::{CustomSwitch, InstallerSwitches, SilentSwitch, SilentWithProgressSwitch},
    },
    locale::{
        Author, Copyright, DefaultLocaleManifest, Description, License, Moniker, PackageName,
        Publisher, ShortDescription, Tag,
    },
    shared::{
        LanguageTag, ManifestType, ManifestVersion, PackageIdentifier, PackageVersion,
        url::{
            CopyrightUrl, DecodedUrl, LicenseUrl, PackageUrl, PublisherSupportUrl, PublisherUrl,
            ReleaseNotesUrl,
        },
    },
    version::VersionManifest,
};

use crate::{
    commands::utils::{
        SPINNER_TICK_RATE, SubmitOption, prompt_existing_pull_request, prompt_submit_option,
        write_changes_to_dir,
    },
    credential::{get_default_headers, handle_token},
    download_file::{download_urls, process_files},
    github::{
        github_client::{GITHUB_HOST, GitHub, WINGET_PKGS_FULL_NAME},
        utils::{get_package_path, pull_request::pr_changes},
    },
    manifests::Manifests,
    prompts::{
        check_prompt, handle_inquire_error,
        list::list_prompt,
        radio_prompt,
        text::{confirm_prompt, optional_prompt, required_prompt},
    },
};

/// 从头开始创建一个新包
#[derive(Parser)]
pub struct NewVersion {
    /// 包的唯一标识符
    #[arg()]
    package_identifier: Option<PackageIdentifier>,

    /// 包的版本
    #[arg(short = 'v', long = "version")]
    package_version: Option<PackageVersion>,

    /// 包安装程序列表
    #[arg(short, long, num_args = 1.., value_hint = clap::ValueHint::Url)]
    urls: Vec<DecodedUrl>,

    #[arg(long)]
    package_locale: Option<LanguageTag>,

    #[arg(long)]
    publisher: Option<Publisher>,

    #[arg(long, value_hint = clap::ValueHint::Url)]
    publisher_url: Option<PublisherUrl>,

    #[arg(long, value_hint = clap::ValueHint::Url)]
    publisher_support_url: Option<PublisherSupportUrl>,

    #[arg(long)]
    package_name: Option<PackageName>,

    #[arg(long, value_hint = clap::ValueHint::Url)]
    package_url: Option<PackageUrl>,

    #[arg(long)]
    moniker: Option<Moniker>,

    #[arg(long)]
    author: Option<Author>,

    #[arg(long)]
    license: Option<License>,

    #[arg(long, value_hint = clap::ValueHint::Url)]
    license_url: Option<LicenseUrl>,

    #[arg(long)]
    copyright: Option<Copyright>,

    #[arg(long, value_hint = clap::ValueHint::Url)]
    copyright_url: Option<CopyrightUrl>,

    #[arg(long)]
    short_description: Option<ShortDescription>,

    #[arg(long)]
    description: Option<Description>,

    #[arg(long, value_hint = clap::ValueHint::Url)]
    release_notes_url: Option<ReleaseNotesUrl>,

    /// 同时下载的安装程序数量
    #[arg(long, default_value_t = NonZeroU8::new(2).unwrap())]
    concurrent_downloads: NonZeroU8,

    /// 添加此包或版本将解决的问题列表
    #[arg(long)]
    resolves: Option<Vec<NonZeroU32>>,

    /// 自动提交拉取请求
    #[arg(short, long)]
    submit: bool,

    /// 调用 Komac 的外部工具名称
    #[arg(long, env = "KOMAC_CREATED_WITH")]
    created_with: Option<String>,

    /// 调用 Komac 的外部工具的 URL
    #[arg(long, env = "KOMAC_CREATED_WITH_URL", value_hint = clap::ValueHint::Url)]
    created_with_url: Option<DecodedUrl>,

    /// 输出清单文件的目录
    #[arg(short, long, env = "OUTPUT_DIRECTORY", value_hint = clap::ValueHint::DirPath)]
    output: Option<Utf8PathBuf>,

    /// 自动打开拉取请求链接
    #[arg(long, env = "OPEN_PR")]
    open_pr: bool,

    /// Run without prompting or submitting
    #[arg(long, env = "DRY_RUN")]
    dry_run: bool,

    /// Skip checking for existing pull requests
    #[arg(long, env)]
    skip_pr_check: bool,

    /// GitHub personal access token with the `public_repo` scope
    #[arg(short, long, env = "GITHUB_TOKEN")]
    token: Option<String>,
}

impl NewVersion {
    pub async fn run(self) -> Result<()> {
        let token = handle_token(self.token.as_deref()).await?;
        let github = GitHub::new(&token)?;
        let client = Client::builder()
            .default_headers(get_default_headers(None))
            .build()?;

        let package_identifier = required_prompt(self.package_identifier)?;

        let versions = github.get_versions(&package_identifier).await.ok();

        let latest_version = versions.as_ref().and_then(BTreeSet::last);

        if let Some(latest_version) = latest_version {
            println!("{package_identifier} 的最新版本是: {latest_version}");
        }

        let manifests =
            latest_version.map(|version| github.get_manifests(&package_identifier, version));

        let package_version = required_prompt(self.package_version)?;

        if let Some(pull_request) = github
            .get_existing_pull_request(&package_identifier, &package_version)
            .await?
        {
            if !(self.skip_pr_check || self.dry_run)
                && !prompt_existing_pull_request(
                    &package_identifier,
                    &package_version,
                    &pull_request,
                )?
            {
                return Ok(());
            }
        }

        let mut urls = self.urls;
        if urls.is_empty() {
            while urls.len() < 1024 {
                let message = format!("{} 安装程序 URL", (urls.len() + 1).to_number());
                let url_prompt = CustomType::<DecodedUrl>::new(&message)
                    .with_error_message("请输入有效的 URL");
                let installer_url = if urls.len() + 1 == 1 {
                    Some(url_prompt.prompt().map_err(handle_inquire_error)?)
                } else {
                    url_prompt
                        .with_help_message("如果没有更多的 URL，请按 ESC")
                        .prompt_skippable()
                        .map_err(handle_inquire_error)?
                };
                if let Some(url) = installer_url {
                    urls.push(url);
                } else {
                    break;
                }
            }
        }

        let github_values = urls
            .iter()
            .find(|url| url.host_str() == Some(GITHUB_HOST))
            .and_then(|url| github.get_all_values_from_url(url));

        let mut files = download_urls(&client, urls, self.concurrent_downloads).await?;
        let mut download_results = process_files(&mut files).await?;

        let mut installers = Vec::new();
        for analyser in &mut download_results.values_mut() {
            let mut installer_switches = InstallerSwitches::default();
            if analyser
                .installers
                .iter()
                .any(|installer| installer.r#type == Some(InstallerType::Exe))
            {
                if confirm_prompt(&format!("{} 是一个便携式应用程序吗?", analyser.file_name))? {
                    for installer in &mut analyser.installers {
                        installer.r#type = Some(InstallerType::Portable);
                    }
                }
                installer_switches.silent = Some(required_prompt::<SilentSwitch>(None)?);
                installer_switches.silent_with_progress =
                    Some(required_prompt::<SilentWithProgressSwitch>(None)?);
            }
            if analyser
                .installers
                .iter()
                .any(|installer| installer.r#type == Some(InstallerType::Portable))
            {
                installer_switches.custom = optional_prompt::<CustomSwitch>(None)?;
            }
            if let Some(zip) = &mut analyser.zip {
                zip.prompt()?;
            }
            let mut analyser_installers = mem::take(&mut analyser.installers);
            for installer in &mut analyser_installers {
                if installer_switches.is_any_some() {
                    installer.switches = Some(installer_switches.clone());
                }
            }
            installers.extend(analyser_installers);
        }
        let default_locale = required_prompt(self.package_locale)?;
        let manifests = match manifests {
            Some(manifests) => Some(manifests.await?),
            None => None,
        };
        let mut installer_manifest = InstallerManifest {
            package_identifier: package_identifier.clone(),
            package_version: package_version.clone(),
            install_modes: if installers
                .iter()
                .any(|installer| installer.r#type == Some(InstallerType::Inno))
            {
                Some(InstallModes::all())
            } else {
                check_prompt::<InstallModes>()?
            },
            success_codes: list_prompt::<InstallerSuccessCode>()?,
            upgrade_behavior: Some(radio_prompt::<UpgradeBehavior>()?),
            commands: list_prompt::<Command>()?,
            protocols: list_prompt::<Protocol>()?,
            file_extensions: if installers
                .iter()
                .all(|installer| installer.file_extensions.is_none())
            {
                list_prompt::<FileExtension>()?
            } else {
                None
            },
            installers,
            manifest_type: ManifestType::Installer,
            ..InstallerManifest::default()
        };

        let mut github_values = match github_values {
            Some(future) => Some(future.await?),
            None => None,
        };
        let default_locale_manifest = DefaultLocaleManifest {
            package_identifier: package_identifier.clone(),
            package_version: package_version.clone(),
            package_locale: default_locale.clone(),
            publisher: match download_results
                .values_mut()
                .find(|analyser| analyser.publisher.is_some())
                .and_then(|analyser| analyser.publisher.take())
            {
                Some(publisher) => publisher,
                None => required_prompt(self.publisher)?,
            },
            publisher_url: optional_prompt(self.publisher_url)?,
            publisher_support_url: optional_prompt(self.publisher_support_url)?,
            author: optional_prompt(self.author)?,
            package_name: match download_results
                .values_mut()
                .find(|analyser| analyser.package_name.is_some())
                .and_then(|analyser| analyser.package_name.take())
            {
                Some(package_name) => package_name,
                None => required_prompt(self.package_name)?,
            },
            package_url: optional_prompt(self.package_url)?,
            license: match github_values
                .as_mut()
                .and_then(|values| values.license.take())
            {
                Some(license) => license,
                None => required_prompt(self.license)?,
            },
            license_url: optional_prompt(self.license_url)?,
            copyright: match download_results
                .values_mut()
                .find(|analyser| analyser.copyright.is_some())
                .and_then(|analyser| analyser.copyright.take())
            {
                Some(copyright) => Some(copyright),
                None => optional_prompt(self.copyright)?,
            },
            copyright_url: optional_prompt(self.copyright_url)?,
            short_description: required_prompt(self.short_description)?,
            description: optional_prompt(self.description)?,
            moniker: optional_prompt(self.moniker)?,
            tags: match github_values
                .as_mut()
                .and_then(|values| values.topics.take())
            {
                Some(topics) => Some(topics),
                None => list_prompt::<Tag>()?,
            },
            release_notes_url: optional_prompt(self.release_notes_url)?,
            manifest_type: ManifestType::DefaultLocale,
            ..DefaultLocaleManifest::default()
        };

        installer_manifest
            .installers
            .iter_mut()
            .filter_map(|installer| installer.apps_and_features_entries.as_mut())
            .flatten()
            .for_each(|entry| entry.deduplicate(&package_version, &default_locale_manifest));

        installer_manifest.reorder_keys(&package_identifier, &package_version);

        let version_manifest = VersionManifest {
            package_identifier: package_identifier.clone(),
            package_version: package_version.clone(),
            default_locale,
            manifest_type: ManifestType::Version,
            manifest_version: ManifestVersion::default(),
        };

        let manifests = Manifests {
            installer: installer_manifest,
            default_locale: default_locale_manifest,
            locales: manifests
                .map(|manifests| manifests.locales)
                .unwrap_or_default(),
            version: version_manifest,
        };

        let package_path = get_package_path(&package_identifier, Some(&package_version), None);
        let mut changes = pr_changes()
            .package_identifier(&package_identifier)
            .manifests(&manifests)
            .package_path(&package_path)
            .maybe_created_with(self.created_with.as_deref())
            .create()?;

        if let Some(output) = self.output.map(|out| out.join(package_path)) {
            write_changes_to_dir(&changes, output.as_path()).await?;
            println!(
                "{} 将所有清单文件写入 {output}",
                "成功".green()
            );
        }

        let submit_option = prompt_submit_option(
            &mut changes,
            self.submit,
            &package_identifier,
            &package_version,
            self.dry_run,
        )?;

        if submit_option == SubmitOption::Exit {
            return Ok(());
        }

        // 创建一个不确定的进度条，以显示正在创建拉取请求
        let pr_progress = ProgressBar::new_spinner().with_message(format!(
            "正在为 {package_identifier} {package_version} 创建拉取请求"
        ));
        pr_progress.enable_steady_tick(SPINNER_TICK_RATE);

        let pull_request_url = github
            .add_version()
            .identifier(&package_identifier)
            .version(&package_version)
            .maybe_versions(versions.as_ref())
            .changes(changes)
            .maybe_issue_resolves(self.resolves)
            .maybe_created_with(self.created_with)
            .maybe_created_with_url(self.created_with_url)
            .send()
            .await?;

        pr_progress.finish_and_clear();

        println!(
            "{} 创建了一个向 {WINGET_PKGS_FULL_NAME} 的拉取请求",
            "成功".green()
        );
        println!("{}", pull_request_url.as_str());

        if self.open_pr {
            open::that(pull_request_url.as_str())?;
        }

        Ok(())
    }
}
