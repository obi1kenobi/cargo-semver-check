#![forbid(unsafe_code)]

pub mod adapter;
mod check_release;
mod query;
mod util;

use std::env;

use clap::{crate_version, AppSettings, Arg, Command};
use termcolor::{ColorChoice, StandardStream};

use crate::{check_release::run_check_release, util::load_rustdoc_from_file};

#[allow(dead_code)]
pub(crate) struct GlobalConfig {
    printing_to_terminal: bool,
    output_writer: StandardStream,
}

impl GlobalConfig {
    fn new() -> Self {
        let printing_to_terminal = atty::is(atty::Stream::Stdout);

        let color_choice = match std::env::var("CARGO_TERM_COLOR").as_deref() {
            Ok("always") => ColorChoice::Always,
            Ok("alwaysansi") => ColorChoice::AlwaysAnsi,
            Ok("auto") => ColorChoice::Auto,
            Ok("never") => ColorChoice::Never,
            Ok(_) | Err(..) => {
                if printing_to_terminal {
                    ColorChoice::Auto
                } else {
                    ColorChoice::Never
                }
            }
        };

        Self {
            printing_to_terminal,
            output_writer: StandardStream::stdout(color_choice),
        }
    }
}

fn main() -> anyhow::Result<()> {
    let matches = Command::new("cargo-semver-checks")
        .bin_name("cargo")
        .version(crate_version!())
        .subcommand(
            Command::new("semver-checks")
                .version(crate_version!())
                .about("Check your crate for semver violations.")
                .subcommand(
                    Command::new("diff-files")
                        .version(crate_version!())
                        .setting(AppSettings::ArgRequiredElseHelp)
                        .arg(
                            Arg::with_name("current_rustdoc_path")
                                .short('c')
                                .long("current")
                                .value_name("CURRENT_RUSTDOC_JSON")
                                .help("The current rustdoc json output to test for semver violations. Required.")
                                .takes_value(true)
                                .required(true)
                        )
                        .arg(
                            Arg::with_name("baseline_rustdoc_path")
                                .short('b')
                                .long("baseline")
                                .value_name("BASELINE_RUSTDOC_JSON")
                                .help("The rustdoc json file to use as a semver baseline. Required.")
                                .takes_value(true)
                                .required(true)
                        )
                )
                .subcommand(
                    Command::new("check-release")
                        .version(crate_version!())
                        .setting(AppSettings::ArgRequiredElseHelp)
                        .arg(
                            Arg::with_name("current_rustdoc_path")
                                .short('c')
                                .long("current")
                                .value_name("CURRENT_RUSTDOC_JSON")
                                .help("The current rustdoc json output to test for semver violations. Required.")
                                .takes_value(true)
                                .required(true)
                        )
                        .arg(
                            Arg::with_name("baseline_rustdoc_path")
                                .short('b')
                                .long("baseline")
                                .value_name("BASELINE_RUSTDOC_JSON")
                                .help("The rustdoc json file to use as a semver baseline. Required.")
                                .takes_value(true)
                                .required(true)
                        )
                )
        ).get_matches();

    // Descend one level: from `cargo semver-checks` to just `semver-checks`.
    let semver_check = matches
        .subcommand_matches("semver-checks")
        .expect("semver-checks is missing");

    let config = GlobalConfig::new();

    if let Some(diff_files) = semver_check.subcommand_matches("diff-files") {
        let current_rustdoc_path: &str = diff_files
            .get_one::<String>("current_rustdoc_path")
            .expect("current_rustdoc_path is required but was not present")
            .as_str();
        let baseline_rustdoc_path: &str = diff_files
            .get_one::<String>("baseline_rustdoc_path")
            .expect("baseline_rustdoc_path is required but was not present")
            .as_str();

        let current_crate = load_rustdoc_from_file(current_rustdoc_path)?;
        let baseline_crate = load_rustdoc_from_file(baseline_rustdoc_path)?;

        return run_check_release(config, current_crate, baseline_crate);
    } else if let Some(check_release) = semver_check.subcommand_matches("check-release") {
        let current_rustdoc_path: &str = check_release
            .get_one::<String>("current_rustdoc_path")
            .expect("current_rustdoc_path is required but was not present")
            .as_str();
        let baseline_rustdoc_path: &str = check_release
            .get_one::<String>("baseline_rustdoc_path")
            .expect("baseline_rustdoc_path is required but was not present")
            .as_str();

        let current_crate = load_rustdoc_from_file(current_rustdoc_path)?;
        let baseline_crate = load_rustdoc_from_file(baseline_rustdoc_path)?;

        return run_check_release(config, current_crate, baseline_crate);
    }

    unreachable!("no commands matched")
}
