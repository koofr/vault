use std::{net::SocketAddr, path::PathBuf, sync::Arc};

use clap::Parser;
use tokio::signal;
use vault_fake_remote::fake_remote::{
    app::{FakeRemoteApp, FakeRemoteAppConfig},
    files::objects::{
        fs_object_provider::FsObjectProvider, memory_object_provider::MemoryObjectProvider,
        object_provider::BoxObjectProvider,
    },
};

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// HTTP server listen addr
    #[arg(long, default_value = "127.0.0.1:3080")]
    http_addr: SocketAddr,

    /// HTTPS server listen addr
    #[arg(long, default_value = "127.0.0.1:3443")]
    https_addr: SocketAddr,

    /// Data path (default empty, in-memory)
    #[arg(long)]
    data_path: Option<PathBuf>,

    /// Default user ID
    #[arg(long, default_value = "b2977f16-4766-4528-a26f-4b0b13bf2c9c")]
    user_id: String,

    /// Default mount ID
    #[arg(long, default_value = "9fd62581-3bad-478a-702b-01937d2bf7f1")]
    mount_id: String,

    /// Default oauth2 access token
    #[arg(long, default_value = "f1fed68a-6b5c-4067-928e-40ed48dd2589")]
    oauth2_access_token: String,

    /// Default oauth2 refresh token
    #[arg(long, default_value = "a126768a-ce0b-4b93-8a9b-809f02f4c000")]
    oauth2_refresh_token: String,

    /// Create a default vault repo (safe box)
    #[arg(long, default_value = "false")]
    create_vault_repo: bool,
}

fn main() {
    let args = Args::parse();

    let mut env_logger_builder = env_logger::Builder::from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );
    let _ = env_logger_builder.try_init();

    let tokio_runtime = Arc::new(tokio::runtime::Runtime::new().unwrap());

    let object_provider: Arc<BoxObjectProvider> = match args.data_path.clone() {
        Some(data_path) => {
            std::fs::create_dir_all(&data_path).unwrap();

            log::info!("Data path: {:?}", data_path);

            Arc::new(Box::new(FsObjectProvider::new(data_path)))
        }
        None => {
            log::info!("Data in memory");

            Arc::new(Box::new(MemoryObjectProvider::new()))
        }
    };

    let config = FakeRemoteAppConfig {
        http_addr: args.http_addr,
        https_addr: args.https_addr,
        object_provider,
        user_id: args.user_id,
        mount_id: args.mount_id,
        oauth2_access_token: args.oauth2_access_token,
        oauth2_refresh_token: args.oauth2_refresh_token,
        create_vault_repo: args.create_vault_repo,
    };

    tokio_runtime.clone().block_on(async move {
        let app = FakeRemoteApp::new(config, tokio_runtime).await;

        app.start().await.unwrap();

        let _ = signal::ctrl_c().await;

        app.stop().await;
    });
}
