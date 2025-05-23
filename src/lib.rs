//! `env-inventory`: A Unified Environment Variable Management Crate
//!
//! This crate provides utilities for easily registering and managing
//! environment variables within your Rust applications. This ensures a
//! centralized approach to handling environment configurations, offering a
//! consistent method of accessing parameters from the environment.
//!
//! Features:
//! - **Unified Access**: Access environment parameters uniformly from anywhere
//!   in the code.
//! - **TOML Integration**: Allows loading of parameters from TOML configuration
//!   files.
//! - **Precedence Handling**: Parameters loaded merge with environment
//!   variables, where the latter takes precedence.
//! - **Registration System**: Variables of interest are registered via the
//!   provided macros, ensuring that you only focus on the ones you need.
//!
//! Usage involves registering variables using the provided macros, and then
//! employing the provided utilities to load and validate these variables either
//! from the environment or TOML files.
//!
//! Note: `dotenv` file support isn't available currently.
//! Note: This crate is still in its early stages and is subject to change.
//! Note: `shell-expansions` (probably using
//! [https://docs.rs/shellexpand/latest/shellexpand/fn.tilde.html](shellexpand))
//! coming soon.

// ce-env/src/lib.rs

#![deny(missing_docs)]
pub extern crate inventory;
extern crate thiserror;
extern crate toml;

use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::Path;
use thiserror::Error;
use toml::Value;

/// Registers one or more environment variables for tracking and validation.
///
/// This macro simplifies the process of registering environment variables that
/// your application depends on. Once registered, you can utilize
/// `env-inventory`'s utilities to load, validate, and manage these environment
/// variables.
///
/// # Examples
///
/// ```rust (ignore)
/// # #[macro_use] extern crate env_inventory;
/// # fn main() {
/// register!("DATABASE_URL", "REDIS_URL", "API_KEY");
/// register!("LOG_LEVEL" => "debug", "CACHE_SIZE" => 1024);
/// # }
/// ```
///
/// The first call registers three environment variables: `DATABASE_URL`,
/// `REDIS_URL`, and `API_KEY`. The second call registers two environment variables
/// with default values: `LOG_LEVEL` with a default of `"debug"`, and `CACHE_SIZE`
/// with a default of `1024`.
///
/// # Parameters
///
/// - `$($var:expr),*`: A comma-separated list of string literals, each
///   representing an environment variable to register.
/// - `$($var:expr => $default:expr),*`: A comma-separated list of pairs, where
///   each pair consists of a string literal representing an environment variable
///   and its default value.
///
/// # Panics
///
/// This macro will panic at compile-time if any of the provided arguments are
/// not string literals or if the pairs don't have the appropriate structure.
#[macro_export]
macro_rules! register {
    ($var:ident) => {
        const _: () = {
            use $crate::RequiredVar;
            $crate::inventory::submit!(RequiredVar::new(stringify!($var)));
        };
    };

    ($var:ident | $err:expr ) => {
        const _: () = {
            use $crate::RequiredVar;
            $crate::inventory::submit!(RequiredVar {
                name: stringify!($var),
                default: None,
                error: Some($err),
                source: file!(),
                priority: $crate::Priority::Library,
            });
        };
    };

    ($($var:ident),* $(,)?) => {
        const _: () = {
            use $crate::RequiredVar;
            $(
                $crate::register!($var);
            )*
        };
    };

    ($($var:ident | $err:expr),* $(,)?) => {
        const _: () => {
            use $crate::RequiredVar;
            $(
                $crate::register!($var | $err);
            )*
        };
    };

    ($var:ident = $default:expr) => {
        const _: () = {
            use $crate::RequiredVar;
            $crate::inventory::submit!(RequiredVar {
                name: stringify!($var),
                default: Some($default),
                error: None,
                source: file!(),
                priority: $crate::Priority::Library,
            });
        };
    };

    ($($var:ident = $default:expr),* $(,)?) => {
        const _: () = {
            use $crate::RequiredVar;
            $(
                $crate::register!($var = $default);
            )*
        };
    };

    ($var:ident = $default:expr; $priority:ident) => {
        const _: () = {
            use $crate::RequiredVar;
            $crate::inventory::submit!(RequiredVar {
                name: stringify!($var),
                default: Some($default),
                error: None,
                source: file!(),
                priority: $crate::Priority::$priority,
            });
        };
    };


    ($($var:ident = $default:expr),* $(,)?; $priority:ident) => {
        const _: () = {
            use $crate::RequiredVar;
            $(
                $crate::register!($var = $default; $priority);
            )*
        };
    };

}

/// Represents the potential errors that can be encountered by the
/// `env-inventory` module.
///
/// This enum provides specific error variants to handle different failure
/// scenarios when working with environment variable loading and validation in
/// the `env-inventory` module. It is designed to give users of the module clear
/// feedback on the nature of the error encountered.
///
/// # Variants
///
/// * `ReadFileError`: Occurs when there's an issue reading a settings file.
/// * `ParseFileError`: Occurs when parsing a settings file fails, possibly due
///   to a malformed structure.
/// * `MissingEnvVars`: Occurs when one or more registered environment variables
///   are not present in either the environment or the settings files.
///
/// # Examples
///
/// ```rust (ignore)
/// # use std::fs;
/// # use env_inventory::EnvInventoryError;
/// fn read_settings(file_path: &str) -> Result<(), EnvInventoryError> {
///     if fs::read(file_path).is_err() {
///         return Err(EnvInventoryError::ReadFileError(file_path.to_string()));
///     }
///     // ... Additional logic ...
///     Ok(())
/// }
/// ```
#[derive(Error, Debug)]
pub enum EnvInventoryError {
    /// Represents a failure to read a settings file.
    ///
    /// Contains a string that provides the path to the file that failed to be
    /// read.
    #[error("Failed to read the settings file at {0}")]
    ReadFileError(String),

    /// Represents a failure to parse a settings file.
    ///
    /// Contains a string that provides the path to the file that failed to be
    /// parsed.
    #[error("Failed to parse the settings file at {0}")]
    ParseFileError(String),

    /// Represents the absence of required environment variables.
    ///
    /// Contains a vector of strings, each representing a missing environment
    /// variable.
    #[error("Missing required environment variables: {0:?}")]
    MissingEnvVars(Vec<String>),

    /// Represents the absence of required environment variables.
    /// variable.
    #[error("Missing required environment variables: {0:?}")]
    MissingEnvVar(String),
}

#[doc(hidden)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Priority {
    /// The default value is from a library or unknown.
    Unknown,
    Library,
    Binary,
}

#[doc(hidden)]
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RequiredVar {
    pub name: &'static str,
    pub default: Option<&'static str>,
    pub error: Option<&'static str>,
    pub source: &'static str,
    pub priority: Priority,
}
impl std::fmt::Debug for RequiredVar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "RequiredVar {{ name: {}, default: {:?}, error: {:?}, source: {}, priority: {:?} }}",
            self.name, self.default, self.error, self.source, self.priority
        )
    }
}

impl std::fmt::Display for RequiredVar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "RequiredVar {{ name: {}, default: {:?}, error: {:?}, source: {}, priority: {:?} }}",
            self.name, self.default, self.error, self.source, self.priority
        )
    }
}

// This macro is used to register the `RequiredVar` struct with the inventory
// system. It allows the `RequiredVar` instances to be collected and managed
// by the inventory crate.
inventory::collect!(RequiredVar);
impl RequiredVar {
    /// Creates a new `RequiredVar` instance at compile time.
    pub const fn new(name: &'static str) -> Self {
        Self {
            name,
            default: None,
            error: None,
            source: "<none>",
            priority: Priority::Library,
        }
    }

    /// Checks if the variable is set in the environment or has a default value.
    pub fn is_set(&self) -> bool {
        // If the variable is set in the environment, or
        // we have a default value, we're good
        env::var(self.name).is_ok() || self.default.is_some()
    }

    /// Gets the value of the variable from the environment or the default.
    pub fn get(&self) -> Option<String> {
        match env::var(self.name) {
            Ok(value) => Some(value),
            Err(_) => match self.default {
                Some(value) => Some(value.to_string()),
                None => None,
            },
        }
    }
}

/// Validates that all registered environment variables are set.
///
/// This function checks if the previously registered environment variables (via
/// the `register!` macro or other means) are present either in the system's
/// environment or the loaded configuration files.
///
/// If any of the registered variables are missing, an
/// `EnvInventoryError::MissingEnvVars` error is returned, containing a list of
/// the missing variables.
///
/// # Parameters
///
/// * `config_paths`: A slice of file paths (as `&str`) pointing to the
///   configuration files that might contain the environment variables. These
///   files are expected to be in TOML format with a dedicated `[env]` section.
/// * `section_name`: The name of the section in the TOML files that contains
///   the environment variables. By default, this is `"env"`.
///
/// # Returns
///
/// * `Ok(())`: If all registered environment variables are found.
/// * `Err(EnvInventoryError)`: If there's an error reading or parsing the
///   config files or if any registered environment variable is missing.
///
/// # Examples
///
/// ```rust (ignore)
/// # use env_inventory::validate_env_vars;
/// let result = validate_env_vars(&["/path/to/settings.conf"], "env");
/// if result.is_err() {
///     eprintln!("Failed to validate environment variables: {:?}", result);
/// }
/// ```
///
/// # Errors
///
/// This function can return the following errors:
/// * `ReadFileError`: If a provided config file cannot be read.
/// * `ParseFileError`: If a provided config file cannot be parsed as TOML or
///   lacks the expected structure.
/// * `MissingEnvVars`: If one or more registered environment variables are
///   missing.

pub fn validate_env_vars() -> Result<(), EnvInventoryError> {
    let missing_vars: HashSet<String> = inventory::iter::<RequiredVar>()
        .filter_map(|var| {
            if var.is_set() {
                None
            } else {
                let msg = if let Some(err) = var.error {
                    format!("{}={}", var.name, err)
                } else {
                    format!("{}={}", var.name, "(missing)")
                };
                Some(msg)
            }
        })
        .collect();
    let mut missing_vars = missing_vars.into_iter().collect::<Vec<String>>();
    missing_vars.sort();
    

    if missing_vars.is_empty() {
        Ok(())
    } else {
        // tracing::warn!("Missing required environment variables: {:?}", missing_vars);
        Err(EnvInventoryError::MissingEnvVars(missing_vars))
    }
}

/// List all the registered environment variables.
/// that are expected from different parts of the application.
pub fn list_all_vars() -> Vec<String> {
    let mut v: Vec<String> = inventory::iter::<RequiredVar>()
        .map(|v| format!("{:#?}", v))
        .collect();
    v.sort();
    v
}

/// Dump all the registered environment variables.

pub fn dump_all_vars() {
    let mut v: Vec<String> = inventory::iter::<RequiredVar>()
        .map(|v| format!("{:#?}", v))
        .collect();
    v.sort();
    dbg!(v);
}

#[doc(hidden)]
pub fn map() -> HashMap<&'static str, String> {
    let mut seen_vars: HashMap<&'static str, String> = HashMap::new();

    for var in inventory::iter::<RequiredVar>() {
        if !seen_vars.contains_key(var.name) {
            if let Some(value) = var.get() {
                seen_vars.insert(var.name, value);
            }
        }
    }
    seen_vars
}

/// Expand all the registered environment variables.
/// that are expected from different parts of the application.
/// So for instance if you have a variable like this:
/// ```rust (ignore)
/// // somewhere
/// register!(TEST_ENV_VAR = "~/test");
/// // elsewhere you do this:
/// register!(LIBDIR = "${TEST_ENV_VAR}/lib");
/// // then you can do this:
///
/// ```
/// then expanded_map will update the env and expand the env.
/// TODO:
/// 1. This should be done in the register macro.
pub fn expanded_map() -> Result<HashMap<String, String>, EnvInventoryError> {
    let mut seen_vars: HashMap<String, String> = HashMap::new();

    for var in inventory::iter::<RequiredVar>() {
        if !seen_vars.contains_key(var.name) {
            if let Some(value) = var.get() {
                let value = shellexpand::full(&value)
                    .map_err(|e| EnvInventoryError::MissingEnvVar(e.to_string()))?
                    .to_string();

                std::env::set_var(var.name, &value);
                seen_vars.insert(var.name.to_string(), value);
            }
        }
    }
    for (key, value) in seen_vars.iter() {
        let value = shellexpand::full(&value)
            .map_err(|e| EnvInventoryError::MissingEnvVar(e.to_string()))?
            .to_string();

        std::env::set_var(key, &value);
    }
    Ok(seen_vars)
}

/// Loads the settings from a TOML file and returns them as a `HashMap`.
pub(crate) fn load_toml_settings<P: AsRef<Path>>(
    path: P,
    section: &str,
) -> Result<HashMap<String, String>, EnvInventoryError> {
    let content = fs::read_to_string(&path)
        .map_err(|_| EnvInventoryError::ReadFileError(path.as_ref().display().to_string()))?;

    let value = content.parse::<Value>().map_err(|e| {
        eprintln!("Error parsing TOML: {}", e);
        EnvInventoryError::ParseFileError(path.as_ref().display().to_string())
    })?;

    let env_section = match value.get(section) {
        Some(env) => env.as_table(),
        None => None,
    };

    let mut settings = HashMap::new();

    if let Some(env_table) = env_section {
        for (key, val) in env_table.iter() {
            if let Some(val_str) = val.as_str() {
                settings.insert(key.clone(), val_str.to_string());
            }
        }
    }
    Ok(settings)
}

/// Loads environment variables from specified configuration files and validates
/// their presence.
///
/// This function goes through the provided list of configuration file paths,
/// merges the environment settings from each file, and ensures that all the
/// registered environment variables are set. If an environment variable
/// is not already present in the system's environment, it will be set using the
/// value from the merged settings.
///
/// Environment variables present in the system's environment take precedence
/// over those in the configuration files.
///
/// # Parameters
///
/// * `config_paths`: A slice containing paths to the configuration files that
///   should be loaded. The files are expected to be in TOML format and have a
///   dedicated section for environment variables.
/// * `section`: The name of the section in the TOML files that contains the
///   environment variables.
///
/// # Returns
///
/// * `Ok(())`: If all registered environment variables are present either in
///   the system's environment or in the merged settings.
/// * `Err(EnvInventoryError)`: If there's an error reading or parsing the
///   config files or if any registered environment variable is missing.
///
/// # Behavior
///
/// The first file in the `config_paths` slice is mandatory and if it can't be
/// read or parsed, an error is immediately returned. Subsequent files are
/// optional, and while they will generate a warning if they cannot be read or
/// parsed, they won't cause the function to return an error.
///
/// After merging the settings from all files and overlaying them on the
/// system's environment variables, the function checks for missing required
/// environment variables and returns an error if any are found.
///
/// # Examples
///
/// ```rust (ignore)
/// # use env_inventory::load_and_validate_env_vars;
/// # use std::path::Path;
/// let paths = [Path::new("/path/to/shipped.conf"), Path::new("/path/to/system.conf")];
/// let result = load_and_validate_env_vars(&paths, "env");
/// if result.is_err() {
///     eprintln!("Failed to load and validate environment variables: {:?}", result);
/// }
/// ```
///
/// # Errors
///
/// This function can return the following errors:
/// * `ReadFileError`: If a provided config file cannot be read.
/// * `ParseFileError`: If a provided config file cannot be parsed as TOML or
///   lacks the expected structure.
/// * `MissingEnvVars`: If one or more registered environment variables are
///   missing.

pub fn load_and_validate_env_vars<P: AsRef<Path>>(
    config_paths: &[P],
    section: &str,
) -> Result<(), EnvInventoryError> {
    let mut merged_settings = HashMap::new();

    for (index, path) in config_paths.iter().enumerate() {
        let settings = load_toml_settings(path.as_ref(), section);

        match settings {
            Ok(current_settings) => {
                // Merge settings with nth file being most significant
                for (key, value) in current_settings.iter() {
                    merged_settings.insert(key.clone(), value.clone());
                }
            }
            Err(e) => {
                if index == 0 {
                    // The first file is mandatory
                    eprintln!(
                        "Error: Could not load settings from {:?}. Reason: {}",
                        path.as_ref(),
                        e
                    );
                    return Err(e);
                } else {
                    // Subsequent files are optional, but let's warn for transparency
                    eprintln!(
                        "Warning: Could not load settings from {:?}. Reason: {}",
                        path.as_ref(),
                        e
                    );
                }
            }
        }
    }

    // let mut missing_vars = Vec::new();

    for var in inventory::iter::<RequiredVar>() {
        // 1) Check if set in env
        if env::var(var.name).is_ok() {
            continue;
        }

        // 2) Check if set in config files
        if let Some(value) = merged_settings.get(var.name) {
            env::set_var(var.name, value);
            continue;
        }

        // 3) Check if set by binary
        let binary_default = inventory::iter::<RequiredVar>()
            .filter(|v| v.name == var.name && v.priority == Priority::Binary)
            .last() // Get the most significant binary default
            .and_then(|v| v.default);

        if let Some(default_value) = binary_default {
            env::set_var(var.name, default_value);
            continue;
        }

        // 4) Check if set by library (with nth library being the most significant)
        let library_default = inventory::iter::<RequiredVar>()
            .filter(|v| v.name == var.name && v.priority == Priority::Library)
            .last() // Get the most significant library default
            .and_then(|v| v.default);

        if let Some(default_value) = library_default {
            env::set_var(var.name, default_value);
            continue;
        }

        // 5) If not set, add to missing
        // missing_vars.push(var.name.to_string());
    }

    expanded_map()?;
    validate_env_vars()

    // if missing_vars.is_empty() {
    //     Ok(())
    // } else {
    //     tracing::warn!("Missing required environment variables: {:?}", missing_vars);
    //     Err(EnvInventoryError::MissingEnvVars(missing_vars))
    // }
}

#[doc(hidden)]
pub fn __old_load_and_validate_env_vars<P: AsRef<Path>>(
    config_paths: &[P],
    section: &str,
) -> Result<(), EnvInventoryError> {
    let mut merged_settings = HashMap::new();

    for (index, path) in config_paths.iter().enumerate() {
        let settings = load_toml_settings(path.as_ref(), section);

        match settings {
            Ok(current_settings) => {
                // Merge settings
                for (key, value) in current_settings.iter() {
                    if !merged_settings.contains_key(key) {
                        merged_settings.insert(key.clone(), value.clone());
                    }
                }
            }
            Err(e) => {
                if index == 0 {
                    // The first file is mandatory
                    return Err(e);
                } else {
                    // Subsequent files are optional, but let's warn for transparency
                    eprintln!(
                        "Warning: Could not load settings from {:?}. Reason: {}",
                        path.as_ref(),
                        e
                    );
                }
            }
        }
    }

    // Override the environment variables with our merged settings if they aren't
    // already set
    for (key, value) in merged_settings.iter() {
        if env::var(key).is_err() {
            env::set_var(key, value);
        }
        let value = env::var(key).unwrap();
        tracing::info!("{} = {}", key, value);
    }
    return validate_env_vars();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use tempfile::tempdir;

    register!(TEST_ENV_VAR = "foo"; Binary);

    #[test]
    fn test_load_single_toml() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("settings.conf");

        fs::write(&file_path, "[env]\nTEST_ENV_VAR = \"test_value\"").unwrap();

        load_and_validate_env_vars(&[file_path], "env").unwrap();
        assert_eq!(env::var("TEST_ENV_VAR").unwrap(), "test_value");
    }

    #[test]
    fn test_merge_priority() {
        let dir = tempdir().unwrap();
        let file_path1 = dir.path().join("settings1.conf");
        let file_path2 = dir.path().join("settings2.conf");
        fs::write(&file_path1, "[env]\nTEST_ENV_VAR = \"value1\"").unwrap();
        fs::write(&file_path2, "[env]\nTEST_ENV_VAR = \"value2\"").unwrap();

        eprintln!("files are in {} directory", dir.path().display());
        load_and_validate_env_vars(&[file_path1, file_path2], "env").unwrap();
        assert_eq!(env::var("TEST_ENV_VAR").unwrap(), "value2");
    }

    #[test]
    fn test_missing_mandatory_config() {
        let dir = tempdir().unwrap();
        let file_path1 = dir.path().join("does_not_exist.conf");
        let file_path2 = dir.path().join("settings.conf");
        fs::write(&file_path2, "[env]\nTEST_ENV_VAR = \"test_value\"").unwrap();

        assert!(load_and_validate_env_vars(&[file_path1, file_path2], "env").is_err());
    }

    #[test]
    fn test_missing_env_vars() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("settings.conf");

        // Write a file without MISSING_VAR
        fs::write(&file_path, "[env]\nSOME_OTHER_VAR = \"some_value\"").unwrap();

        // Ensure the environment variable isn't set before the test
        env::remove_var("MISSING_VAR");

        // Register MISSING_VAR as a required environment variable
        register!(MISSING_VAR);

        // Since MISSING_VAR isn't in the environment and also isn't in the TOML files,
        // the function should return an error.
        assert!(load_and_validate_env_vars(&[file_path], "env").is_err());
    }

    #[test]
    fn test_present_env_vars() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("settings.conf");

        // Write a file with PRESENT_VAR
        fs::write(
            &file_path,
            r#"
        [env]
        PRESENT_VAR = "present_value"
        MISSING_VAR = "missing_value"
        TEST_ENV_VAR = "test_value"
        "#,
        )
        .unwrap();

        // Ensure the environment variable isn't set before the test
        env::remove_var("PRESENT_VAR");

        // Register PRESENT_VAR as a required environment variable
        register!(PRESENT_VAR = "default_value"; Binary);

        // Since PRESENT_VAR is in the TOML file, the function should run without errors
        load_and_validate_env_vars(&[file_path], "env").unwrap();
    }
}
