use std::collections::HashMap;

use ini::Ini;
use slug::slugify;
use thiserror::Error;

use crate::{rclone::obscure::obscure, utils::path_utils::normalize_path};

use super::obscure::reveal;

#[derive(Debug, PartialEq, Eq)]
pub struct Config {
    pub name: Option<String>,
    pub path: String,
    pub password: String,
    pub salt: Option<String>,
}

#[derive(Error, Debug, Clone, PartialEq)]
#[error("parse config failed: {0}")]
pub struct ParseConfigError(String);

pub fn parse_config(config_str: &str) -> Result<Config, ParseConfigError> {
    let i = Ini::load_from_str(&config_str).map_err(|e| ParseConfigError(e.to_string()))?;

    let sections: Vec<(Option<&str>, HashMap<&str, &str>)> = i
        .iter()
        .map(|(section_name, prop)| {
            let prop_map: HashMap<_, _> = prop.iter().collect();

            (section_name, prop_map)
        })
        .collect();

    let crypt_sections: Vec<_> = sections
        .iter()
        .filter(|(_, props)| props.get("type").filter(|typ| **typ == "crypt").is_some())
        .collect();

    if crypt_sections.len() != 1 {
        return Err(ParseConfigError(String::from(
            "expected config to have exactly 1 crypt section",
        )));
    }

    let (section_name, props) = crypt_sections.first().unwrap();

    let remote = props
        .get("remote")
        .ok_or_else(|| ParseConfigError(String::from("missing remote property")))?;
    let path = remote
        .split(":")
        .nth(1)
        .ok_or_else(|| ParseConfigError(String::from("remote missing path")))?;
    let path =
        normalize_path(path).map_err(|_| ParseConfigError(String::from("invalid remote path")))?;
    let obscured_password = props
        .get("password")
        .ok_or_else(|| ParseConfigError(String::from("missing password property")))?;
    let password = reveal(obscured_password)
        .map_err(|e| ParseConfigError(format!("failed to reveal password: {}", e.to_string())))?;
    let obscured_salt = props.get("password2").map(|salt| salt.to_string());
    let salt = match obscured_salt {
        Some(obscured_salt) => Some(reveal(&obscured_salt).map_err(|e| {
            ParseConfigError(format!("failed to reveal password2: {}", e.to_string()))
        })?),
        None => None,
    };

    Ok(Config {
        name: section_name.map(|name| name.to_string()),
        path: path.to_owned(),
        password: password.to_string(),
        salt,
    })
}

pub fn generate_config(config: &Config) -> String {
    let mut i = Ini::new();

    let section_name = slugify(config.name.as_deref().unwrap_or("vault"));
    let remote = format!("koofr:{}", config.path);
    let obscured_password = obscure(&config.password).unwrap();

    {
        i.with_section(Some(&section_name))
            .set("type", "crypt")
            .set("remote", remote)
            .set("password", obscured_password);

        if let Some(salt) = config.salt.as_deref() {
            i.with_section(Some(&section_name))
                .set("password2", obscure(&salt).unwrap());
        }
    }

    let mut out = Vec::new();

    i.write_to(&mut out).unwrap();

    String::from_utf8(out).unwrap()
}

#[cfg(test)]
pub mod tests {
    use regex::Regex;

    use super::{generate_config, parse_config, Config};

    #[test]
    fn test_parse_config() {
        assert_eq!(
            parse_config("[vault-name]\ntype=crypt\nremote=koofr:/Vault\npassword=YMRulMcUAOo9raAGnYdie57EWnDFi_N283rEVw\npassword2=8lJriKxI5ersT8JUSYG6jm2H-1q2HiiM\n")
                .unwrap(),
            Config {
                name: Some(String::from("vault-name")),
                path: String::from("/Vault"),
                password: String::from("testpassword"),
                salt: Some(String::from("testsalt")),
            }
        );

        assert_eq!(
            parse_config("[vault-name]\ntype=crypt\nremote=koofr:/Vault\npassword=YMRulMcUAOo9raAGnYdie57EWnDFi_N283rEVw\n")
                .unwrap(),
            Config {
                name: Some(String::from("vault-name")),
                path: String::from("/Vault"),
                password: String::from("testpassword"),
                salt: None,
            }
        );

        assert_eq!(
            parse_config("[vault-name]\ntype=crypt\nremote=koofr:Vault\npassword=YMRulMcUAOo9raAGnYdie57EWnDFi_N283rEVw\n")
                .unwrap(),
            Config {
                name: Some(String::from("vault-name")),
                path: String::from("/Vault"),
                password: String::from("testpassword"),
                salt: None,
            }
        );

        assert_eq!(
            parse_config("[vault-name]\ntype=crypt\nremote=koofr:/\npassword=YMRulMcUAOo9raAGnYdie57EWnDFi_N283rEVw\n")
                .unwrap(),
            Config {
                name: Some(String::from("vault-name")),
                path: String::from("/"),
                password: String::from("testpassword"),
                salt: None,
            }
        );

        assert_eq!(
            parse_config("[vault-name]\ntype=crypt\nremote=koofr:\npassword=YMRulMcUAOo9raAGnYdie57EWnDFi_N283rEVw\n")
                .unwrap(),
            Config {
                name: Some(String::from("vault-name")),
                path: String::from("/"),
                password: String::from("testpassword"),
                salt: None,
            }
        );

        assert_eq!(
            parse_config("[vault-1]\ntype=crypt\nremote=koofr:/Vault\npassword=YMRulMcUAOo9raAGnYdie57EWnDFi_N283rEVw\n\n[vault-2]\ntype=crypt\nremote=koofr:/Vault\npassword=YMRulMcUAOo9raAGnYdie57EWnDFi_N283rEVw\n")
                .unwrap_err()
                .to_string(),
            "parse config failed: expected config to have exactly 1 crypt section"
        );

        assert_eq!(
            parse_config("").unwrap_err().to_string(),
            "parse config failed: expected config to have exactly 1 crypt section"
        );

        assert_eq!(
            parse_config("[vault-name]\ntype=crypt\npassword=YMRulMcUAOo9raAGnYdie57EWnDFi_N283rEVw\npassword2=8lJriKxI5ersT8JUSYG6jm2H-1q2HiiM\n")
                .unwrap_err()
                .to_string(),
            "parse config failed: missing remote property"
        );

        assert_eq!(
            parse_config("[vault-name]\ntype=crypt\nremote=koofr:..\npassword=YMRulMcUAOo9raAGnYdie57EWnDFi_N283rEVw\npassword2=8lJriKxI5ersT8JUSYG6jm2H-1q2HiiM\n")
                .unwrap_err()
                .to_string(),
            "parse config failed: invalid remote path"
        );

        assert_eq!(
            parse_config("[vault-name]\ntype=crypt\nremote=koofr:/Vault\n")
                .unwrap_err()
                .to_string(),
            "parse config failed: missing password property"
        );

        assert_eq!(
            parse_config("[vault-name]\ntype=crypt\nremote=koofr:/Vault\npassword=testpassword\n")
                .unwrap_err()
                .to_string(),
            "parse config failed: failed to reveal password: obscure error: input too short when revealing password - is it obscured?"
        );

        assert_eq!(
            parse_config("[vault-name]\ntype=crypt\nremote=koofr:/Vault\npassword=YMRulMcUAOo9raAGnYdie57EWnDFi_N283rEVw\npassword2=testsalt\n")
                .unwrap_err()
                .to_string(),
            "parse config failed: failed to reveal password2: obscure error: input too short when revealing password - is it obscured?"
        );
    }

    #[test]
    fn test_generate_config() {
        let config = generate_config(&Config {
            name: Some(String::from("Vault name")),
            path: String::from("/Vault"),
            password: String::from("testpassword"),
            salt: Some(String::from("testsalt")),
        });
        let expected =
            "^\\[vault-name\\]\ntype=crypt\nremote=koofr:/Vault\npassword=.*\npassword2=.*\n$";
        assert!(Regex::new(expected).unwrap().is_match(&config));

        let config = generate_config(&Config {
            name: None,
            path: String::from("/Vault"),
            password: String::from("testpassword"),
            salt: Some(String::from("testsalt")),
        });
        let expected =
            "^\\[vault\\]\ntype=crypt\nremote=koofr:/Vault\npassword=.*\npassword2=.*\n$";
        assert!(Regex::new(expected).unwrap().is_match(&config));

        let config = generate_config(&Config {
            name: Some(String::from("Vault name")),
            path: String::from("/Vault"),
            password: String::from("testpassword"),
            salt: None,
        });
        let expected = "^\\[vault-name\\]\ntype=crypt\nremote=koofr:/Vault\npassword=.*\n$";
        assert!(Regex::new(expected).unwrap().is_match(&config));
    }
}
