use futures::{io::Cursor, AsyncReadExt};
use std::{str, sync::Arc};
use xsalsa20poly1305::XSalsa20Poly1305;

use crate::types::{DecryptedName, DecryptedPath, EncryptedName, EncryptedPath};

use super::{
    cipher_keys::{derive_keys, DerivedKeys},
    constants::{DATA_KEY_LEN, NAME_CIPHER_BLOCK_SIZE, NAME_KEY_LEN},
    data_cipher::get_data_cipher,
    decrypt_reader::DecryptReader,
    encrypt_reader::EncryptReader,
    errors::DecryptFilenameError,
    name_cipher::{
        decrypt_filename, decrypt_path, encrypt_filename, encrypt_path, get_name_cipher,
    },
    nonce::Nonce,
};

pub struct Cipher {
    name_key: [u8; NAME_KEY_LEN],
    name_tweak: [u8; NAME_CIPHER_BLOCK_SIZE],
    data_cipher: Arc<XSalsa20Poly1305>,
}

impl Cipher {
    pub fn new(password: &str, salt: Option<&str>) -> Self {
        let DerivedKeys {
            data_key,
            name_key,
            name_tweak,
        } = derive_keys(password, salt);

        Self::with_keys(data_key, name_key, name_tweak)
    }

    pub fn with_keys(
        data_key: [u8; DATA_KEY_LEN],
        name_key: [u8; NAME_KEY_LEN],
        name_tweak: [u8; NAME_CIPHER_BLOCK_SIZE],
    ) -> Self {
        let data_cipher = get_data_cipher(&data_key);

        Self {
            name_key,
            name_tweak,
            data_cipher: Arc::new(data_cipher),
        }
    }

    pub fn encrypt_filename(&self, plaintext: &DecryptedName) -> EncryptedName {
        EncryptedName(encrypt_filename(
            get_name_cipher(&self.name_key, &self.name_tweak),
            &plaintext.0,
        ))
    }

    pub fn encrypt_path(&self, plaintext: &DecryptedPath) -> EncryptedPath {
        EncryptedPath(encrypt_path(
            get_name_cipher(&self.name_key, &self.name_tweak),
            &plaintext.0,
        ))
    }

    pub fn decrypt_filename(
        &self,
        ciphertext: &EncryptedName,
    ) -> Result<DecryptedName, DecryptFilenameError> {
        decrypt_filename(
            get_name_cipher(&self.name_key, &self.name_tweak),
            &ciphertext.0,
        )
        .map(DecryptedName)
    }

    pub fn decrypt_path(
        &self,
        ciphertext: &EncryptedPath,
    ) -> Result<DecryptedPath, DecryptFilenameError> {
        decrypt_path(
            get_name_cipher(&self.name_key, &self.name_tweak),
            &ciphertext.0,
        )
        .map(DecryptedPath)
    }

    pub fn encrypt_reader<R>(&self, reader: R) -> EncryptReader<R> {
        let nonce = Nonce::new_random().unwrap();

        EncryptReader::new(reader, self.data_cipher.clone(), nonce)
    }

    pub async fn encrypt_data(
        &self,
        data: &[u8],
        out: &mut Vec<u8>,
    ) -> Result<usize, std::io::Error> {
        let reader = Cursor::new(data);

        self.encrypt_reader(reader).read_to_end(out).await
    }

    pub fn decrypt_reader<R>(&self, reader: R) -> DecryptReader<R> {
        DecryptReader::new(reader, self.data_cipher.clone())
    }

    pub async fn decrypt_data(
        &self,
        data: &[u8],
        out: &mut Vec<u8>,
    ) -> Result<usize, std::io::Error> {
        let reader = Cursor::new(data);

        self.decrypt_reader(reader).read_to_end(out).await
    }
}

#[cfg(test)]
mod tests {
    use futures::executor::block_on;

    use crate::types::{DecryptedName, DecryptedPath, EncryptedName, EncryptedPath};

    use super::Cipher;

    #[test]
    fn test_encrypt_filename() {
        // tested with rclone 1.60
        let cipher = Cipher::new("testpassword", None);

        assert_eq!(
            cipher.encrypt_filename(&DecryptedName("testfilename".into())),
            EncryptedName("mvedi866srqc97sl5948oaej2g".into())
        );
        assert_eq!(
            cipher.encrypt_filename(&DecryptedName("testfilenametestfilename".into())),
            EncryptedName("7dpcdrb8vdgnmm0g0n5behih3pltha7hllb1mncolkrqgp3p8b2g".into())
        );
        assert_eq!(
            cipher.encrypt_filename(&DecryptedName(
                "testfilenametestfilenametestfilename".into()
            )),
            EncryptedName(
                "ok5l5eohh73jdulluibldf7grqa9r4q9jekamuqkt7is8fdjsn4f32cukfrnagu35bdsc63i9pqos"
                    .into()
            )
        );
        assert_eq!(
            cipher.encrypt_filename(&DecryptedName("testfilenametestfilenametestfilenametestfilename".into())),
            EncryptedName("hvrag3t30d3av1c31lp7j7klq95j12ru932ujf4hmaf6b4h2f2ooro1gnprne2itnibamt67h7j5a2bn1c0gkqni2n4pb17937rg22g".into())
        );
        assert_eq!(
            cipher.encrypt_filename(&DecryptedName("testfilenametestfilenametestfilenametestfilenametestfilename".into())),
            EncryptedName("013kcoiml8e3017c34132bdri58a77qea5i5f56npucna727c9tttepfe468e1aj1dmr0aqmn6rtbe2e3j8cgt7qr2rpfrpr4p12vj8".into())
        );
        assert_eq!(
            cipher.encrypt_filename(&DecryptedName("testfilenametestfilenametestfilenametestfilenametestfilenametestfilename".into())),
            EncryptedName("shcqqn35vr6ehsn5dqft11fpmr7em9krg0hvlbts0a5jb1nok9pc706c5jrjt0kug67oto6rt2o7bpqidtddvj16p2js37cgautb0856f7cnk6b4jluu3rvii0q4s87e".into())
        );
        assert_eq!(
            cipher.encrypt_filename(&DecryptedName("testfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilename".into())),
            EncryptedName("jhqr2n0782nqst11m4r5u7k6i1knb9lottugfng4m9l645ts99cr7eggld663llijgsqgrt2mqjm8b5asfb422drg3tnmthv5oj8vt0gkoura72hudp381r62si4les65hpm7jppvf0t9lgf40ime30cvs".into())
        );
        assert_eq!(
            cipher.encrypt_filename(&DecryptedName("testfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilename".into())),
            EncryptedName("5vf0sm8ao57u2k44cpsg8u40uev5e0s4jor1qk7lm351c8m4uroglehuqtlqk93kukq4r1spahkk7bnsigre5up5mlm9s4ssgl3colsn6gcqodvbluj0d2ub7jnbvqqdiutbokm6a27ko4idef590kc70t998kegekv86ij24u3p8ao9k840".into())
        );
        assert_eq!(
            cipher.encrypt_filename(&DecryptedName("testfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilename".into())),
            EncryptedName("i0eqc8m8rsb2192altkfkhh387tunp8lcm14omfbmjneuejjipncpl760rrlfakh0dikhct7i63d84gqsca6kdint24fcfp7mhjfng7hpat8s60jo328n61oborkuqjr8qp81vt5p8ogsnj95sc4p8u1kh64k3130tnicstnvsmrmgpc646g".into())
        );
        assert_eq!(
            cipher.encrypt_filename(&DecryptedName("testfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilename".into())),
            EncryptedName("05oun5jfto8bd68aceui7cgjg44hoi0m8lv3shlqu21vro8ii1o7p4b2bg3mq21qlvj7jsj4n8smrrsc8h5be2s715cu6dj1g1o7jmju62uingea7v9q1sh3dkr77v3pfd14lu1tjpfq9b8qt8dsoiq7dukhpjhv9f8se91j0p69qrkmbf1cucht23ka043v3o9jcug272gjg".into())
        );
        assert_eq!(
            cipher.encrypt_filename(&DecryptedName("testfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilename".into())),
            EncryptedName("s8a8636o3oo1g95i5eh8cg2emaodulttad0460n83btqcukgsrfuna8kjoshtdis1nkh2v30hs0sas90443d59tqofvcsi1r7pd3eoi23mivj3bf3k5vl508doaqghh40t9n74ibhe22jc5n9buq7tn7kj897tjmhd7du3vqc1s8t97drmj7tekuq9f3nb3gmsfufv53adtth9g9qkpicendfl8umro0pk4asv8".into())
        );
        assert_eq!(
            cipher.encrypt_filename(&DecryptedName("testfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilename".into())),
            EncryptedName("aih8hkkjcirtgsg067bg7g9ait5ets26p1vkctjfvmit3h6kun49cjabq01s2vrq9ia2k1q453f1uimjunf98qaja570as68irjdlhtir2vjlvfl6lmj5b18mmb3la3g8f5hhg5bf3kascg93mta0hd32blmur08deg59safibo75hvk51rtgf45r9o6v29qc4gi3kkv89k9rtlf1r4qm5ughavlqigr4ri640nhhf8b12ntidod6tv2jajaomb1".into())
        );
    }

    #[test]
    fn test_encrypt_path() {
        let cipher = Cipher::new("testpassword", None);

        assert_eq!(
            cipher.encrypt_path(&DecryptedPath("/".into())),
            EncryptedPath("/".into())
        );
        assert_eq!(
            cipher.encrypt_path(&DecryptedPath("/testfilename".into())),
            EncryptedPath("/mvedi866srqc97sl5948oaej2g".into())
        );
        assert_eq!(
            cipher.encrypt_path(&DecryptedPath("/testfilename/testfilename".into())),
            EncryptedPath("/mvedi866srqc97sl5948oaej2g/mvedi866srqc97sl5948oaej2g".into())
        );
    }

    #[test]
    fn test_decrypt_filename() {
        // tested with rclone 1.60
        let cipher = Cipher::new("testpassword", None);

        assert_eq!(
            cipher
                .decrypt_filename(&EncryptedName("mvedi866srqc97sl5948oaej2g".into()))
                .unwrap(),
            DecryptedName("testfilename".into())
        );
        assert_eq!(
            cipher
                .decrypt_filename(&EncryptedName(
                    "7dpcdrb8vdgnmm0g0n5behih3pltha7hllb1mncolkrqgp3p8b2g".into()
                ))
                .unwrap(),
            DecryptedName("testfilenametestfilename".into())
        );
        assert_eq!(
            cipher
                .decrypt_filename(&EncryptedName(
                    "ok5l5eohh73jdulluibldf7grqa9r4q9jekamuqkt7is8fdjsn4f32cukfrnagu35bdsc63i9pqos"
                        .into()
                ))
                .unwrap(),
            DecryptedName("testfilenametestfilenametestfilename".into())
        );
        assert_eq!(
            cipher.decrypt_filename(&EncryptedName("hvrag3t30d3av1c31lp7j7klq95j12ru932ujf4hmaf6b4h2f2ooro1gnprne2itnibamt67h7j5a2bn1c0gkqni2n4pb17937rg22g".into())).unwrap(),
            DecryptedName("testfilenametestfilenametestfilenametestfilename".into())
        );
        assert_eq!(
            cipher.decrypt_filename(&EncryptedName("013kcoiml8e3017c34132bdri58a77qea5i5f56npucna727c9tttepfe468e1aj1dmr0aqmn6rtbe2e3j8cgt7qr2rpfrpr4p12vj8".into())).unwrap(),
            DecryptedName("testfilenametestfilenametestfilenametestfilenametestfilename".into())
        );
        assert_eq!(
            cipher.decrypt_filename(&EncryptedName("shcqqn35vr6ehsn5dqft11fpmr7em9krg0hvlbts0a5jb1nok9pc706c5jrjt0kug67oto6rt2o7bpqidtddvj16p2js37cgautb0856f7cnk6b4jluu3rvii0q4s87e".into())).unwrap(),
            DecryptedName("testfilenametestfilenametestfilenametestfilenametestfilenametestfilename".into())
        );
        assert_eq!(
            cipher.decrypt_filename(&EncryptedName("jhqr2n0782nqst11m4r5u7k6i1knb9lottugfng4m9l645ts99cr7eggld663llijgsqgrt2mqjm8b5asfb422drg3tnmthv5oj8vt0gkoura72hudp381r62si4les65hpm7jppvf0t9lgf40ime30cvs".into())).unwrap(),
            DecryptedName("testfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilename".into())
        );
        assert_eq!(
            cipher.decrypt_filename(&EncryptedName("5vf0sm8ao57u2k44cpsg8u40uev5e0s4jor1qk7lm351c8m4uroglehuqtlqk93kukq4r1spahkk7bnsigre5up5mlm9s4ssgl3colsn6gcqodvbluj0d2ub7jnbvqqdiutbokm6a27ko4idef590kc70t998kegekv86ij24u3p8ao9k840".into())).unwrap(),
            DecryptedName("testfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilename".into())
        );
        assert_eq!(
            cipher.decrypt_filename(&EncryptedName("i0eqc8m8rsb2192altkfkhh387tunp8lcm14omfbmjneuejjipncpl760rrlfakh0dikhct7i63d84gqsca6kdint24fcfp7mhjfng7hpat8s60jo328n61oborkuqjr8qp81vt5p8ogsnj95sc4p8u1kh64k3130tnicstnvsmrmgpc646g".into())).unwrap(),
            DecryptedName("testfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilename".into())
        );
        assert_eq!(
            cipher.decrypt_filename(&EncryptedName("05oun5jfto8bd68aceui7cgjg44hoi0m8lv3shlqu21vro8ii1o7p4b2bg3mq21qlvj7jsj4n8smrrsc8h5be2s715cu6dj1g1o7jmju62uingea7v9q1sh3dkr77v3pfd14lu1tjpfq9b8qt8dsoiq7dukhpjhv9f8se91j0p69qrkmbf1cucht23ka043v3o9jcug272gjg".into())).unwrap(),
            DecryptedName("testfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilename".into())
        );
        assert_eq!(
            cipher.decrypt_filename(&EncryptedName("s8a8636o3oo1g95i5eh8cg2emaodulttad0460n83btqcukgsrfuna8kjoshtdis1nkh2v30hs0sas90443d59tqofvcsi1r7pd3eoi23mivj3bf3k5vl508doaqghh40t9n74ibhe22jc5n9buq7tn7kj897tjmhd7du3vqc1s8t97drmj7tekuq9f3nb3gmsfufv53adtth9g9qkpicendfl8umro0pk4asv8".into())).unwrap(),
            DecryptedName("testfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilename".into())
        );
        assert_eq!(
            cipher.decrypt_filename(&EncryptedName("aih8hkkjcirtgsg067bg7g9ait5ets26p1vkctjfvmit3h6kun49cjabq01s2vrq9ia2k1q453f1uimjunf98qaja570as68irjdlhtir2vjlvfl6lmj5b18mmb3la3g8f5hhg5bf3kascg93mta0hd32blmur08deg59safibo75hvk51rtgf45r9o6v29qc4gi3kkv89k9rtlf1r4qm5ughavlqigr4ri640nhhf8b12ntidod6tv2jajaomb1".into())).unwrap(),
            DecryptedName("testfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilename".into())
        );
    }

    #[test]
    fn test_decrypt_path() {
        let cipher = Cipher::new("testpassword", None);

        assert_eq!(
            cipher.decrypt_path(&EncryptedPath("/".into())).unwrap(),
            DecryptedPath("/".into())
        );
        assert_eq!(
            cipher
                .decrypt_path(&EncryptedPath("/mvedi866srqc97sl5948oaej2g".into()))
                .unwrap(),
            DecryptedPath("/testfilename".into())
        );
        assert_eq!(
            cipher
                .decrypt_path(&EncryptedPath(
                    "/mvedi866srqc97sl5948oaej2g/mvedi866srqc97sl5948oaej2g".into()
                ))
                .unwrap(),
            DecryptedPath("/testfilename/testfilename".into())
        );
    }

    #[test]
    fn test_encrypt_data() {
        block_on(async {
            // tested with rclone 1.60
            let cipher = Cipher::new("testpassword", None);

            let mut encrypted = Vec::new();

            let res = cipher
                .encrypt_data("testdata".as_bytes(), &mut encrypted)
                .await;

            assert_eq!(res.unwrap(), 56);

            let mut decrypted = Vec::new();

            let res = cipher.decrypt_data(&encrypted, &mut decrypted).await;

            assert_eq!(res.unwrap(), 8);

            assert_eq!(std::str::from_utf8(&decrypted).unwrap(), "testdata");
        })
    }

    #[test]
    fn test_decrypt_data() {
        block_on(async {
            // tested with rclone 1.60
            let cipher = Cipher::new("testpassword", None);

            let encrypted = vec![
                82, 67, 76, 79, 78, 69, 0, 0, 209, 27, 246, 23, 134, 105, 131, 148, 3, 49, 228, 74,
                200, 43, 245, 170, 123, 102, 24, 137, 45, 77, 53, 115, 206, 216, 221, 4, 40, 177,
                52, 14, 5, 190, 84, 192, 246, 157, 207, 154, 11, 178, 94, 181, 135, 59, 240, 115,
            ];

            let mut decrypted = Vec::new();

            let res = cipher.decrypt_data(&encrypted, &mut decrypted).await;

            assert_eq!(res.unwrap(), 8);

            assert_eq!(std::str::from_utf8(&decrypted).unwrap(), "testdata");
        })
    }
}
