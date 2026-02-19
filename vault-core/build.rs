use std::{
    env, fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use vault_intl::{CatalogFormatter, LanguageIdentifier};

#[derive(Debug, Deserialize)]
struct Locale {
    locale: LanguageIdentifier,
    name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct IntlLocale {
    pub locale: LanguageIdentifier,
    pub name: String,
    pub messages_json: String,
}

fn main() {
    let locales_path = Path::new("src/intl/locales/locales.json");
    println!("cargo:rerun-if-changed={locales_path:?}");

    let locales_json = fs::read_to_string(locales_path).expect("failed to read locales.json");
    let locales: Vec<Locale> = serde_json::from_str(&locales_json).expect("invalid locales.json");
    if locales.is_empty() {
        panic!("invalid locales.json: expected at least one locale");
    }

    let out_dir = env::var("OUT_DIR").expect("missing OUT_DIR");
    let generated_path = Path::new(&out_dir).join("src/intl/locales/locales.min.json");
    let generated_dir = generated_path
        .parent()
        .expect("missing generated output directory");
    fs::create_dir_all(generated_dir).expect("failed to create output directory");

    let locales = locales
        .iter()
        .map(|locale| {
            let messages_path =
                PathBuf::from(format!("src/intl/locales/{}/compiled.json", locale.locale));

            (locale, messages_path)
        })
        .collect::<Vec<_>>();

    for (_, messages_path) in locales.iter() {
        println!("cargo:rerun-if-changed={messages_path:?}");
    }

    let mut locales_with_messages = Vec::with_capacity(locales.len());

    for (locale, messages_path) in locales.iter() {
        // Minify messages json
        let messages_json = fs::read_to_string(&messages_path)
            .unwrap_or_else(|err| panic!("failed to read {messages_path:?}: {err}"));
        let messages_json_value: serde_json::Value = serde_json::from_str(&messages_json)
            .unwrap_or_else(|err| panic!("invalid {messages_path:?}: {err}"));
        let messages_json = serde_json::to_string(&messages_json_value)
            .unwrap_or_else(|err| panic!("failed to minify {messages_path:?}: {err}"));

        let locale = IntlLocale {
            locale: locale.locale.clone(),
            name: locale.name.clone(),
            messages_json: messages_json,
        };

        // Build the catalog formatter to ensure plural rules exist during build
        // time, not only in runtime
        CatalogFormatter::from_str(&locale.locale, &locale.messages_json).unwrap();

        locales_with_messages.push(locale);
    }

    let merged_json = serde_json::to_string(&locales_with_messages)
        .expect("failed to serialize locales with messages");

    fs::write(generated_path, merged_json).expect("failed to write generated locales json");
}
