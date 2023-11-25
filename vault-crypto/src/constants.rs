/// Same as rclone
/// https://github.com/rclone/rclone/blob/7be9855a706d1e09504f17949a90c54cd56fb2a5/backend/crypt/cipher.go#L58
pub const DEFAULT_SALT: &[u8] = &[
    0xA8, 0x0D, 0xF4, 0x3A, 0x8F, 0xBD, 0x03, 0x08, 0xA7, 0xCA, 0xB8, 0x3E, 0x58, 0x1F, 0x86, 0xB1,
];

pub const DATA_KEY_LEN: usize = 32;
pub const NAME_KEY_LEN: usize = 32;
pub const NAME_CIPHER_BLOCK_SIZE: usize = 16;
pub const KEY_LEN: usize = DATA_KEY_LEN + NAME_KEY_LEN + NAME_CIPHER_BLOCK_SIZE;

pub const FILE_MAGIC: &[u8] = b"RCLONE\x00\x00";
pub const FILE_MAGIC_SIZE: usize = FILE_MAGIC.len();
pub const FILE_NONCE_SIZE: usize = 24;
pub const FILE_HEADER_SIZE: usize = FILE_MAGIC_SIZE + FILE_NONCE_SIZE;

/// The size, in bytes, of a poly1305 authenticator.
pub const BLOCK_HEADER_SIZE: usize = 16;
pub const BLOCK_DATA_SIZE: usize = 64 * 1024;
pub const BLOCK_SIZE: usize = BLOCK_HEADER_SIZE + BLOCK_DATA_SIZE;
