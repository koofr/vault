// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;

use futures::{channel::oneshot, FutureExt, TryFutureExt};
use tauri::{api::dialog::FileDialogBuilder, RunEvent};
use vault_core::oauth2::OAuth2Config;
use vault_desktop_server::{
    app::app,
    encryption::Encryption,
    file_handlers::FileHandlers,
    init_secure_storage::{init_file_secure_storage, init_keyring_secure_storage},
};
use vault_native::vault::build_vault;
use vault_web_api::web_vault_base::WebVaultBase;

struct TauriState {
    pub port: u16,
    pub app_secret: String,
}

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

    let (secure_storage, secure_storage_error) =
        match std::env::var("VAULT_SECURE_STORAGE").as_deref() {
            Ok("file") => init_file_secure_storage(&app_id),
            _ => init_keyring_secure_storage(&app_id),
        };

    let tokio_runtime = Arc::new(tokio::runtime::Runtime::new().unwrap());

    let (encryption, app_secret) = Encryption::random().unwrap();
    let encryption = Arc::new(encryption);

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
        pick_files: Some(Arc::new(Box::new(|| {
            let (sender, receiver) = oneshot::channel();

            FileDialogBuilder::new().pick_files(|res| {
                let _ = sender.send(res);
            });

            receiver.unwrap_or_else(|_| None).boxed()
        }))),
        pick_dirs: Some(Arc::new(Box::new(|| {
            let (sender, receiver) = oneshot::channel();

            FileDialogBuilder::new().pick_folders(|res| {
                let _ = sender.send(res);
            });

            receiver.unwrap_or_else(|_| None).boxed()
        }))),
        save_file: Some(Arc::new(Box::new(|name| {
            let (sender, receiver) = oneshot::channel();

            let builder = FileDialogBuilder::new().set_file_name(&name);

            #[cfg(target_os = "windows")]
            let builder = match vault_core::utils::name_utils::name_to_ext(&name) {
                Some(ext) => builder.add_filter(format!("File (.{})", ext), &[ext]),
                None => builder,
            };

            builder.save_file(|res| {
                let _ = sender.send(res);
            });

            receiver.unwrap_or_else(|_| None).boxed()
        }))),
    });

    tauri::async_runtime::spawn(app(
        port,
        web_vault,
        tokio_runtime,
        encryption,
        file_handlers,
    ));

    tauri::Builder::default()
        .manage(TauriState { port, app_secret })
        .invoke_handler(tauri::generate_handler![
            get_desktop_server_url,
            get_app_secret
        ])
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|_app, event| match event {
            RunEvent::Ready => {
                fix_current_dir();
            }
            _ => {}
        });
}

#[tauri::command]
fn get_desktop_server_url(state: tauri::State<TauriState>) -> Result<String, String> {
    Ok(format!("http://127.0.0.1:{}", state.port))
}

#[tauri::command]
fn get_app_secret(state: tauri::State<TauriState>) -> Result<String, String> {
    Ok(state.app_secret.clone())
}

fn fix_current_dir() {
    // fix login (open::that) on Linux with AppImage
    // https://github.com/AppImage/AppImageKit/issues/1293#issuecomment-1800047206
    if let Ok(current_dir) = std::env::current_dir() {
        if current_dir.starts_with("/tmp/") {
            if let Ok(home) = std::env::var("HOME") {
                let _ = std::env::set_current_dir(home);
            }
        }
    }
}
