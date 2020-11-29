use clap::Clap;
use regex::Regex;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::io::{self, Read};
use std::iter;
use std::process::Command;
use yaml_rust::yaml::Hash as YamlHash;
use yaml_rust::{Yaml, YamlEmitter, YamlLoader};

const BASED_ON_STYLE: &str = "BasedOnStyle";

#[derive(Clap)]
struct Arguments {
    #[clap(long, default_value = "clang-format")]
    clang_format_executable: OsString,
    config_file: Option<OsString>,
}

fn run_executable(executable: impl AsRef<OsStr>, args: impl IntoIterator<Item = impl AsRef<OsStr>>) -> String {
    String::from_utf8(Command::new(executable).args(args).output().unwrap().stdout).unwrap()
}

fn get_style_names(clang_format_executable: impl AsRef<OsStr>) -> Box<[String]> {
    let help_text = run_executable(clang_format_executable, &["--help"]);
    let regex = Regex::new(r"^\s*(\w+(?:, \w+)*)\.$").unwrap();

    let mut result = help_text.lines().find_map(|line| regex.captures(line)).unwrap()[1]
        .split(", ")
        .map(str::to_string)
        .collect::<Box<_>>();

    result.sort_unstable();

    result
}

fn parse_single_config(config_text: &str) -> YamlHash {
    YamlLoader::load_from_str(&config_text)
        .unwrap()
        .pop()
        .and_then(Yaml::into_hash)
        .unwrap()
}

fn get_style_config(clang_format_executable: impl AsRef<OsStr>, style: &str) -> YamlHash {
    let config_text = run_executable(clang_format_executable, &["--dump-config", "--style", style]);

    parse_single_config(&config_text)
}

fn merge_yaml_hash(hash: &YamlHash, base_hash: &YamlHash) -> YamlHash {
    hash.iter()
        .filter_map(|(key, value)| {
            base_hash
                .get(key)
                .map_or_else(
                    || Some(value.clone()),
                    |base_value| {
                        if let Yaml::Hash(value_hash) = value {
                            let merged_hash = merge_yaml_hash(value_hash, base_value.as_hash().unwrap());

                            if merged_hash.is_empty() {
                                None
                            } else {
                                Some(Yaml::Hash(merged_hash))
                            }
                        } else if base_value == value {
                            None
                        } else {
                            Some(value.clone())
                        }
                    },
                )
                .map(|merged_value| (key.clone(), merged_value))
        })
        .collect()
}

fn simplify_single_config(config: &YamlHash, style_name: String, style_config: &YamlHash) -> YamlHash {
    let mut simplified_config =
        iter::once((Yaml::String(BASED_ON_STYLE.to_string()), Yaml::String(style_name))).collect::<YamlHash>();

    simplified_config.extend(merge_yaml_hash(config, style_config));

    simplified_config
}

fn simplify_config(config: YamlHash, styles: impl IntoIterator<Item = (String, YamlHash)>) -> YamlHash {
    if config.contains_key(&Yaml::String(BASED_ON_STYLE.to_string())) {
        config
    } else {
        styles
            .into_iter()
            .map(|(name, style_config)| simplify_single_config(&config, name, &style_config))
            .min_by_key(YamlHash::len)
            .unwrap()
    }
}

fn main() {
    let Arguments {
        clang_format_executable,
        config_file,
    } = Arguments::parse();

    let config = parse_single_config(&config_file.map_or_else(
        || {
            let mut config_text = String::new();

            io::stdin().read_to_string(&mut config_text).unwrap();

            config_text
        },
        |config_file| fs::read_to_string(config_file).unwrap(),
    ));

    let style_names = get_style_names(&clang_format_executable);

    let simplified_config = simplify_config(
        config,
        style_names.into_vec().into_iter().map(|style_name| {
            let style = get_style_config(&clang_format_executable, &style_name);

            (style_name, style)
        }),
    );

    let mut simplified_config_text = String::new();

    YamlEmitter::new(&mut simplified_config_text)
        .dump(&Yaml::Hash(simplified_config))
        .unwrap();

    print!("{}", simplified_config_text.strip_prefix("---\n").unwrap());
}
