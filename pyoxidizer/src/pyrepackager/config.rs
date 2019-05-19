// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use serde::Deserialize;

// TOML config file parsing.
#[derive(Debug, Deserialize)]
struct PythonDistribution {
    local_path: Option<String>,
    url: Option<String>,
    sha256: String,
}

#[allow(non_snake_case)]
fn TRUE() -> bool {
    true
}

#[allow(non_snake_case)]
fn FALSE() -> bool {
    false
}

#[allow(non_snake_case)]
fn ZERO() -> i64 {
    0
}

#[derive(Debug, Deserialize)]
struct PythonConfig {
    #[serde(default = "TRUE")]
    dont_write_bytecode: bool,
    #[serde(default = "TRUE")]
    ignore_environment: bool,
    #[serde(default = "TRUE")]
    no_site: bool,
    #[serde(default = "TRUE")]
    no_user_site_directory: bool,
    #[serde(default = "ZERO")]
    optimize_level: i64,
    program_name: Option<String>,
    stdio_encoding: Option<String>,
    #[serde(default = "FALSE")]
    unbuffered_stdio: bool,
    write_modules_directory_env: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum PythonPackaging {
    #[serde(rename = "stdlib-extensions-policy")]
    StdlibExtensionsPolicy {
        // TODO make this an enum.
        policy: String,
    },

    #[serde(rename = "stdlib-extensions-explicit-includes")]
    StdlibExtensionsExplicitIncludes {
        #[serde(default)]
        includes: Vec<String>,
    },

    #[serde(rename = "stdlib-extensions-explicit-excludes")]
    StdlibExtensionsExplicitExcludes {
        #[serde(default)]
        excludes: Vec<String>,
    },

    #[serde(rename = "stdlib")]
    Stdlib {
        #[serde(default = "ZERO")]
        optimize_level: i64,
        #[serde(default = "TRUE")]
        exclude_test_modules: bool,
        #[serde(default = "TRUE")]
        include_source: bool,
    },
    #[serde(rename = "virtualenv")]
    Virtualenv {
        path: String,
        #[serde(default = "ZERO")]
        optimize_level: i64,
        #[serde(default)]
        excludes: Vec<String>,
        #[serde(default = "TRUE")]
        include_source: bool,
    },
    #[serde(rename = "package-root")]
    PackageRoot {
        path: String,
        packages: Vec<String>,
        #[serde(default = "ZERO")]
        optimize_level: i64,
        #[serde(default)]
        excludes: Vec<String>,
        #[serde(default = "TRUE")]
        include_source: bool,
    },
    #[serde(rename = "filter-file-include")]
    FilterFileInclude { path: String },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "mode")]
pub enum RunMode {
    #[serde(rename = "repl")]
    Repl {},
    #[serde(rename = "module")]
    Module { module: String },
    #[serde(rename = "eval")]
    Eval { code: String },
}

#[derive(Debug, Deserialize)]
struct ParsedConfig {
    python_distribution: PythonDistribution,
    python_config: PythonConfig,
    python_packages: Vec<PythonPackaging>,
    python_run: RunMode,
}

#[derive(Debug)]
pub struct Config {
    pub dont_write_bytecode: bool,
    pub ignore_environment: bool,
    pub no_site: bool,
    pub no_user_site_directory: bool,
    pub optimize_level: i64,
    pub program_name: String,
    pub python_distribution_path: Option<String>,
    pub python_distribution_url: Option<String>,
    pub python_distribution_sha256: String,
    pub stdio_encoding_name: Option<String>,
    pub stdio_encoding_errors: Option<String>,
    pub unbuffered_stdio: bool,
    pub python_packaging: Vec<PythonPackaging>,
    pub run: RunMode,
    pub write_modules_directory_env: Option<String>,
}

pub fn parse_config(data: &[u8]) -> Config {
    let config: ParsedConfig = toml::from_slice(&data).unwrap();

    let optimize_level = match config.python_config.optimize_level {
        0 => 0,
        1 => 1,
        2 => 2,
        value => panic!("illegal optimize_level {}; value must be 0, 1, or 2", value),
    };

    let program_name = match config.python_config.program_name {
        Some(value) => value,
        None => String::from("undefined"),
    };

    let python_distribution_path = config.python_distribution.local_path;
    let python_distribution_url = config.python_distribution.url;
    let python_distribution_sha256 = config.python_distribution.sha256;

    let (stdio_encoding_name, stdio_encoding_errors) = match config.python_config.stdio_encoding {
        Some(value) => {
            let values: Vec<&str> = value.split(':').collect();
            (Some(values[0].to_string()), Some(values[1].to_string()))
        }
        None => (None, None),
    };

    let mut have_stdlib_extensions_policy = false;
    let mut have_stdlib = false;

    for packaging in &config.python_packages {
        match packaging {
            PythonPackaging::StdlibExtensionsPolicy { .. } => {
                have_stdlib_extensions_policy = true;
            }
            PythonPackaging::Stdlib { .. } => {
                have_stdlib = true;
            }
            _ => {}
        }
    }

    if !have_stdlib_extensions_policy {
        panic!("no `type = \"stdlib-extensions-policy\"` entry in `[[python_packages]]`");
    }

    if !have_stdlib {
        panic!("no `type = \"stdlib\"` entry in `[[python_packages]]`");
    }

    Config {
        dont_write_bytecode: config.python_config.dont_write_bytecode,
        ignore_environment: config.python_config.ignore_environment,
        no_site: config.python_config.no_site,
        no_user_site_directory: config.python_config.no_user_site_directory,
        optimize_level,
        program_name,
        python_distribution_path,
        python_distribution_url,
        python_distribution_sha256,
        stdio_encoding_name,
        stdio_encoding_errors,
        unbuffered_stdio: config.python_config.unbuffered_stdio,
        python_packaging: config.python_packages,
        run: config.python_run,
        write_modules_directory_env: config.python_config.write_modules_directory_env,
    }
}
