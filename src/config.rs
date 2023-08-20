use std::{env, fs::File};

use serde::{Deserialize, Serialize};

const CONFIG_DIR: &str = "CONFIG_DIR";

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct ConfigRecipeError;

#[derive(Serialize, Deserialize, Debug)]
pub enum Environment {
    Local,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub environment: Environment,
    pub basepath: String,
    pub filepath: String,
}

impl Config {
    pub fn read(s: String) -> Result<Self, ConfigRecipeError> {
        let file = File::open(s).map_or(Err(ConfigRecipeError), Ok);
        serde_json::from_reader(file?).map_or(Err(ConfigRecipeError), Ok)
    }

    #[must_use]
    pub fn path() -> String {
        env::var(CONFIG_DIR).map_or_else(
            |_| {
                env::current_dir()
                    .expect("Could not get `CONFIG_DIR` or current directory.")
                    .join(String::from("config.json"))
                    .to_str()
                    .expect("Could not get `CONFIG_DIR` to current directory.")
                    .into()
            },
            |path| path,
        )
    }
}

#[cfg(test)]
mod tests {
    use std::{
        env::{remove_var, set_var},
        io::{Seek, SeekFrom, Write},
    };

    use tempfile::NamedTempFile;

    use super::*;

    #[test]
    fn test_given_bad_path_when_read_then_error() {
        assert_eq!(
            Config::read(String::from("bad")).err(),
            Some(ConfigRecipeError)
        );
    }

    #[test]
    fn test_given_path_with_bad_content_when_read_then_error() {
        let mut tmpfile = NamedTempFile::new().unwrap();
        write!(tmpfile, "Hello World!").unwrap();
        tmpfile.seek(SeekFrom::Start(0)).unwrap();
        let old_path: &str = tmpfile.path().as_os_str().to_str().expect("not found");
        assert_eq!(
            Config::read(String::from(old_path)).unwrap_err(),
            ConfigRecipeError
        );
    }

    #[test]
    fn test_given_path_and_valid_config_when_read_then_config_initialized() {
        let json = r#"
        {
            "environment": "Local",
            "basepath": ".",
            "filepath": "./assets/carne_asada.dat"
        }"#;
        let mut tmpfile = NamedTempFile::new().unwrap();
        write!(tmpfile, "{json}").unwrap();
        tmpfile.seek(SeekFrom::Start(0)).unwrap();
        let old_path = tmpfile.path().as_os_str().to_str().expect("not found");
        let conf = Config::read(String::from(old_path)).unwrap();
        assert!(matches!(conf.environment, Environment::Local));
    }

    #[test]
    fn test_given_no_env_var_when_path_then_current_directory() {
        remove_var(CONFIG_DIR);
        assert!(Config::path().ends_with("config.json"));
    }

    #[test]
    fn test_given_env_var_when_path_then_current_directory() {
        set_var(CONFIG_DIR, "hello, world!");
        assert_eq!(Config::path(), "hello, world!");
        remove_var(CONFIG_DIR);
    }
}
