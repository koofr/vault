use std::{
    net::SocketAddr,
    path::PathBuf,
    sync::{Arc, RwLock},
};

use clap::Parser;
use tokio::signal;
use vault_core::remote::models;
use vault_fake_remote::fake_remote::{
    self, actions,
    context::Context,
    eventstream,
    files::{self, service::FilesService},
    state::FakeRemoteState,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Listen addr
    #[arg(long, default_value = "127.0.0.1:3443")]
    addr: SocketAddr,

    /// Data path
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

    let data_path = args
        .data_path
        .clone()
        .unwrap_or_else(|| std::env::temp_dir().join("vault-fake-remote-data"));

    std::fs::create_dir_all(&data_path).unwrap();

    log::info!("Data path: {:?}", data_path);

    tokio_runtime.clone().block_on(async move {
        let args = args;

        let state = Arc::new(RwLock::new(FakeRemoteState::default()));
        let eventstream_listeners = Arc::new(eventstream::Listeners::new());
        let files_service = Arc::new(FilesService::new(
            state.clone(),
            eventstream_listeners.clone(),
            data_path,
        ));

        init_state(&state, &files_service, &args).await;

        let fake_remote_server = fake_remote::FakeRemoteServer::new(
            state.clone(),
            files_service.clone(),
            eventstream_listeners.clone(),
            Some(args.addr),
            fake_remote::CERT_PEM.to_owned(),
            fake_remote::KEY_PEM.to_owned(),
            tokio_runtime.clone(),
        );

        fake_remote_server.start().await.unwrap();

        let _ = signal::ctrl_c().await;

        fake_remote_server.stop().await;
    });
}

async fn init_state(state: &RwLock<FakeRemoteState>, files_service: &FilesService, args: &Args) {
    let user_id = args.user_id.clone();
    let mount_id = args.mount_id.clone();

    {
        let mut state = state.write().unwrap();

        actions::create_user(
            &mut state,
            files_service,
            Some(user_id.clone()),
            Some(mount_id.clone()),
        );

        state.default_user_id = Some(user_id.clone());

        state
            .oauth2_access_tokens
            .insert(args.oauth2_access_token.clone(), user_id.to_owned());
        state
            .oauth2_refresh_tokens
            .insert(args.oauth2_refresh_token.clone(), user_id.to_owned());
    }

    if args.create_vault_repo {
        let context = Context {
            user_id: user_id.clone(),
            user_agent: None,
        };

        files_service
            .create_dir(
                &context,
                &mount_id,
                &files::Path::root(),
                files::Name("My safe box".into()),
            )
            .await
            .unwrap();

        {
            let mut state = state.write().unwrap();

            actions::create_vault_repo(
                &context,
                &mut state,
                models::VaultRepoCreate {
                    mount_id: mount_id.clone(),
                    path: "/My safe box".into(),
                    salt: Some("salt".into()),
                    password_validator: "ad3238a5-5fc7-4b8f-9575-88c69c0c91cd".into(),
                    password_validator_encrypted: "v2:UkNMT05FAABVyJmka7FKh8CKL2AtIZc1xiZk-SO5GeuZPnHvw0ehM1dENa4iBCyPEf50da9V2XvL5CjpZlUle1lifEHtaRy9YHoFLHtiq1PCAqYY".into(),
                },
            )
            .unwrap();
        }
    }
}
