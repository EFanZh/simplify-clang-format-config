use crate::language::Language;
use clap::Clap;
use regex::Regex;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::io::{self, Read};
use std::iter;
use std::process::Command;
use yaml_rust::yaml::Hash as YamlHash;
use yaml_rust::{Yaml, YamlEmitter, YamlLoader};

mod language;

const BASED_ON_STYLE_KEY: &str = "BasedOnStyle";
const LANGUAGE_KEY: &str = "Language";

#[derive(Clap)]
struct Arguments {
    #[clap(long, default_value = "clang-format")]
    clang_format_executable: OsString,
    config_file: Option<OsString>,
}

fn run_executable(executable: &OsStr, args: impl IntoIterator<Item = impl AsRef<OsStr>>) -> String {
    String::from_utf8(Command::new(executable).args(args).output().unwrap().stdout).unwrap()
}

fn get_style_names(clang_format_executable: &OsStr) -> Box<[String]> {
    let help_text = run_executable(clang_format_executable, &["--help"]);
    let style_list_regex = Regex::new(r"^\s*(\w+(?:, \w+)*)\.$").unwrap();

    let mut result = help_text
        .lines()
        .find_map(|line| style_list_regex.captures(line))
        .unwrap()[1]
        .split(", ")
        .map(str::to_string)
        .collect::<Box<_>>();

    result.sort_unstable();

    result
}

fn parse_configurations(config_text: &str) -> Box<[YamlHash]> {
    YamlLoader::load_from_str(config_text)
        .unwrap()
        .into_iter()
        .map(|yaml| yaml.into_hash().unwrap())
        .collect()
}

fn get_style_config(clang_format_executable: &OsStr, language: Option<Language>, style: &str) -> YamlHash {
    let config_text = language.map_or_else(
        || run_executable(clang_format_executable, &["--dump-config", "--style", style]),
        |language| {
            run_executable(
                clang_format_executable,
                &[
                    "--assume-filename",
                    language.get_file_extension(),
                    "--dump-config",
                    "--style",
                    style,
                ],
            )
        },
    );

    parse_configurations(&config_text).into_vec().pop().unwrap()
}

fn simplify_yaml_hash(hash: &YamlHash, base_hash: &YamlHash) -> YamlHash {
    hash.iter()
        .filter_map(|(key, value)| {
            base_hash
                .get(key)
                .map_or_else(
                    || {
                        value.as_hash().map_or_else(
                            || Some(value.clone()),
                            |value_hash| {
                                if value_hash.is_empty() {
                                    None
                                } else {
                                    Some(Yaml::Hash(value_hash.clone()))
                                }
                            },
                        )
                    },
                    |base_value| {
                        if let Yaml::Hash(value_hash) = value {
                            let simplified_hash = simplify_yaml_hash(value_hash, base_value.as_hash().unwrap());

                            if simplified_hash.is_empty() {
                                None
                            } else {
                                Some(Yaml::Hash(simplified_hash))
                            }
                        } else if base_value == value {
                            None
                        } else {
                            Some(value.clone())
                        }
                    },
                )
                .map(|simplified_value| (key.clone(), simplified_value))
        })
        .collect()
}

fn simplify_single_config(
    config: &YamlHash,
    language: Option<Language>,
    style_name: &str,
    style_config: &YamlHash,
) -> YamlHash {
    let mut simplified_config = language.map_or_else(YamlHash::new, |language_name| {
        iter::once((
            Yaml::String(LANGUAGE_KEY.to_string()),
            Yaml::String(language_name.get_name().to_string()),
        ))
        .collect()
    });

    simplified_config.insert(
        Yaml::String(BASED_ON_STYLE_KEY.to_string()),
        Yaml::String(style_name.to_string()),
    );

    simplified_config.extend(simplify_yaml_hash(config, style_config));

    simplified_config
}

fn simplify_config<'a>(
    config: YamlHash,
    language: Option<Language>,
    styles: impl IntoIterator<Item = (&'a str, YamlHash)>,
) -> YamlHash {
    if config.contains_key(&Yaml::String(BASED_ON_STYLE_KEY.to_string())) {
        config
    } else {
        styles
            .into_iter()
            .map(|(style_name, style_config)| simplify_single_config(&config, language, style_name, &style_config))
            .min_by_key(YamlHash::len)
            .unwrap()
    }
}

fn main() {
    let Arguments {
        clang_format_executable,
        config_file,
    } = Arguments::parse();

    let configs = parse_configurations(&config_file.map_or_else(
        || {
            let mut config_text = String::new();

            io::stdin().read_to_string(&mut config_text).unwrap();

            config_text
        },
        |config_file| fs::read_to_string(config_file).unwrap(),
    ));

    let style_names = get_style_names(&clang_format_executable);
    let mut result = String::new();
    let language_key = Yaml::String(LANGUAGE_KEY.to_string());

    for config in configs.into_vec() {
        let mut yaml_emitter = YamlEmitter::new(&mut result);

        let language = config
            .get(&language_key)
            .and_then(Yaml::as_str)
            .and_then(|language_value| language_value.parse().ok());

        let simplified_config = simplify_config(
            config,
            language,
            style_names.iter().map(|style_name| {
                (
                    style_name.as_str(),
                    get_style_config(&clang_format_executable, language, &style_name),
                )
            }),
        );

        yaml_emitter.dump(&Yaml::Hash(simplified_config)).unwrap();

        result.push('\n');
    }

    if let Some(content) = result.strip_prefix("---\n") {
        print!("{}", content);
    }
}
