use std::sync::Arc;

use vault_core::oauth2::OAuth2Config;
use vault_desktop_server::{
    app::app,
    encryption::Encryption,
    file_handlers::FileHandlers,
    init_secure_storage::{init_file_secure_storage, init_keyring_secure_storage},
};
use vault_native::vault::build_vault;
use vault_web_api::web_vault_base::WebVaultBase;

fn main() {
    let port = 1421;

    let base_url = String::from("https://app.koofr.net");
    // let base_url = String::from("https://127.0.0.1:3443");
    let oauth2_auth_base_url = String::from("https://app.koofr.net");
    // let oauth2_auth_base_url = String::from("http://127.0.0.1:3080");
    let oauth2_client_id = String::from("7ZEK2BNCEVYEJIZC5OR3TR6PQDUJ4NP3");
    let oauth2_client_secret =
        String::from("VWTMENEWUYWH6G523CEV5CWOCHH7FMECW36PPQENOASYYZOQJOSGQXSR2Y62N3HB");
    let oauth2_redirect_uri = format!("http://127.0.0.1:{}/oauth2callback", port);
    let user_agent = String::from("vault-desktop");
    let app_id = String::from("koofr-vault");
    let app_secret = String::from("XrwBl00MUbeAZ4QBW2F+YDFBv80f2kes49VDx7wUs7Y=");

    let (secure_storage, secure_storage_error) =
        match std::env::var("VAULT_SECURE_STORAGE").as_deref() {
            Ok("file") => init_file_secure_storage(&app_id),
            _ => init_keyring_secure_storage(&app_id),
        };

    let tokio_runtime = Arc::new(tokio::runtime::Runtime::new().unwrap());

    let encryption = Arc::new(Encryption::new_with_key_str(&app_secret).unwrap());

    let oauth2_config = OAuth2Config {
        base_url: base_url.clone(),
        auth_base_url: oauth2_auth_base_url,
        client_id: oauth2_client_id,
        client_secret: oauth2_client_secret,
        redirect_uri: oauth2_redirect_uri,
    };

    let (vault, _, _) = build_vault(
        base_url,
        user_agent,
        oauth2_config,
        secure_storage,
        tokio_runtime.clone(),
    );

    if let Some(err) = secure_storage_error {
        vault.notifications_show(err);
    }

    let web_vault = WebVaultBase::new(vault);

    web_vault.load();

    let file_handlers = Arc::new(FileHandlers {
        pick_files: None,
        pick_dirs: None,
        save_file: None,
    });

    tokio_runtime.clone().block_on(async move {
        app(port, web_vault, tokio_runtime, encryption, file_handlers).await
    });
}
