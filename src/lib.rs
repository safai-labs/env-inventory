//! This module provides a way to get parameters from the environment.
//! All parameters are registered here and then can be accessed from anywhere
//! in the code.
//! This unifies the way we get parameters from the environment.
//! Get parameters from dotenv file and environment variables.
//! We gather all the parameters from the dotenv file and environment variables
//! and then merge them together, with the environment file taking precedence.

// ce-env/src/lib.rs

extern crate inventory;
extern crate toml;
extern crate thiserror;

use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use toml::Value;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EnvInventoryError {
    #[error("Failed to read the settings file at {0}")]
    ReadFileError(String),

    #[error("Failed to parse the settings file at {0}")]
    ParseFileError(String),

    #[error("Missing required environment variables: {0:?}")]
    MissingEnvVars(Vec<String>),
}


#[derive(Debug, Clone)]
pub struct RequiredVar {
    pub var_name: &'static str,
    // pub default: Option<&'static str>,
}

inventory::collect!(RequiredVar);

impl RequiredVar {
    pub const fn new(var_name: &'static str, _default: Option<&'static str>) -> Self {
        Self { var_name }
    }
    pub fn is_set(&self) -> bool {
        env::var(self.var_name).is_ok()
    }
}

pub fn validate_env_vars() -> Result<(), EnvInventoryError> {
    let missing_vars: Vec<String> = inventory::iter::<RequiredVar>()
        .filter_map(|var| {
            if var.is_set() {
                None
            } else {
                Some(var.var_name.to_string())
            }
        })
        .collect();

    if missing_vars.is_empty() {
        Ok(())
    } else {
        type E = EnvInventoryError;
        Err(E::MissingEnvVars(missing_vars))
    }
}
pub(crate) fn load_toml_settings<P: AsRef<Path>>(path: P, section: &str) -> Result<HashMap<String, String>, EnvInventoryError> {
    let content = fs::read_to_string(&path)
        .map_err(|_| EnvInventoryError::ReadFileError(path.as_ref().display().to_string()))?;

    let value = content.parse::<Value>()
        .map_err(|_| EnvInventoryError::ParseFileError(path.as_ref().display().to_string()))?;

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


/// loads the settings from the settings file (toml) 
/// and sets the environment variables
pub fn load_and_validate_env_vars<P: AsRef<Path>>(config_paths: &[P], section: &str)
    -> Result<(), EnvInventoryError>
{
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
                    eprintln!("Warning: Could not load settings from {:?}. Reason: {}", path.as_ref(), e);
                }
            }
        }
    }

    // Override the environment variables with our merged settings if they aren't already set
    for (key, value) in merged_settings.iter() {
        if env::var(key).is_err() {
            eprintln!("Setting {} to {}", key, value);
            env::set_var(key, value);
        }
    }

    let missing_vars: Vec<String> = inventory::iter::<RequiredVar>()
        .filter_map(|var| if var.is_set() { None } else { Some(var.var_name.to_string()) })
        .collect();

    if missing_vars.is_empty() {
        Ok(())
    } else {
        Err(EnvInventoryError::MissingEnvVars(missing_vars))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use tempfile::tempdir;

    inventory::submit!(RequiredVar::new("TEST_ENV_VAR", None));

    #[test]
    fn test_load_single_toml() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("settings.conf");
        eprintln!("dir: {:?}", file_path);

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

        load_and_validate_env_vars(&[file_path2, file_path1], "env").unwrap();
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
        inventory::submit!(RequiredVar::new("MISSING_VAR", None));

        // Since MISSING_VAR isn't in the environment and also isn't in the TOML files,
        // the function should return an error.
        assert!(load_and_validate_env_vars(&[file_path], "env").is_err());
    }
    #[test]
    fn test_present_env_vars() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("settings.conf");
    
        // Write a file with PRESENT_VAR
        fs::write(&file_path, r#"
        [env]
        PRESENT_VAR = "present_value"
        MISSING_VAR = "missing_value"
        TEST_ENV_VAR = "test_value" 
        "#).unwrap();
    
        // dump the file
        let content = fs::read_to_string(&file_path).unwrap();
        eprintln!("content:\n{}", content);

        // Ensure the environment variable isn't set before the test
        env::remove_var("PRESENT_VAR");
    
        // Register PRESENT_VAR as a required environment variable
        inventory::submit!(RequiredVar::new("PRESENT_VAR", None));
    
        // Since PRESENT_VAR is in the TOML file, the function should run without errors
        load_and_validate_env_vars(&[file_path], "env").unwrap();
    }
    
}
