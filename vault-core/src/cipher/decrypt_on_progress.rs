use std::sync::{Arc, Mutex};

use vault_crypto::data_cipher::decrypt_size;

pub fn decrypt_on_progress(
    encrypted_on_progress: Box<dyn Fn(usize) + Send + Sync>,
) -> Box<dyn Fn(usize) + Send + Sync> {
    struct OnProgressState {
        pub encrypted_bytes: i64,
        pub decrypted_bytes: i64,
    }

    let state = Arc::new(Mutex::new(OnProgressState {
        encrypted_bytes: 0,
        decrypted_bytes: 0,
    }));

    Box::new(move |n: usize| {
        let mut state = state.lock().unwrap();

        state.encrypted_bytes += n as i64;

        if let Ok(bytes) = decrypt_size(state.encrypted_bytes) {
            if bytes > state.decrypted_bytes {
                encrypted_on_progress((bytes - state.decrypted_bytes) as usize);
            }
            state.decrypted_bytes = bytes;
        }
    })
}
