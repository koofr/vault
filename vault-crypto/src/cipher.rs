use std::{
    io::{Cursor, Read},
    str,
    sync::Arc,
};
use xsalsa20poly1305::XSalsa20Poly1305;

use super::{
    cipher_keys::{derive_keys, DerivedKeys},
    constants::{DATA_KEY_LEN, NAME_CIPHER_BLOCK_SIZE, NAME_KEY_LEN},
    data_cipher::get_data_cipher,
    decrypt_reader::{AsyncDecryptReader, SyncDecryptReader},
    encrypt_reader::{AsyncEncryptReader, SyncEncryptReader},
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

    pub fn encrypt_filename(&self, plaintext: &str) -> String {
        encrypt_filename(
            get_name_cipher(&self.name_key, &self.name_tweak),
            &plaintext,
        )
    }

    pub fn encrypt_path(&self, plaintext: &str) -> String {
        encrypt_path(
            get_name_cipher(&self.name_key, &self.name_tweak),
            &plaintext,
        )
    }

    pub fn decrypt_filename(&self, ciphertext: &str) -> Result<String, DecryptFilenameError> {
        decrypt_filename(
            get_name_cipher(&self.name_key, &self.name_tweak),
            &ciphertext,
        )
    }

    pub fn decrypt_path(&self, ciphertext: &str) -> Result<String, DecryptFilenameError> {
        decrypt_path(
            get_name_cipher(&self.name_key, &self.name_tweak),
            &ciphertext,
        )
    }

    pub fn encrypt_reader_async<R>(&self, reader: R) -> AsyncEncryptReader<R> {
        let nonce = Nonce::new_random().unwrap();

        AsyncEncryptReader::new(reader, self.data_cipher.clone(), nonce)
    }

    pub fn encrypt_reader_sync<R>(&self, reader: R) -> SyncEncryptReader<R> {
        let nonce = Nonce::new_random().unwrap();

        SyncEncryptReader::new(reader, self.data_cipher.clone(), nonce)
    }

    pub fn encrypt_data(&self, data: &[u8], out: &mut Vec<u8>) -> Result<usize, std::io::Error> {
        let reader = Cursor::new(data);

        self.encrypt_reader_sync(reader).read_to_end(out)
    }

    pub fn decrypt_reader_async<R>(&self, reader: R) -> AsyncDecryptReader<R> {
        AsyncDecryptReader::new(reader, self.data_cipher.clone())
    }

    pub fn decrypt_reader_sync<R>(&self, reader: R) -> SyncDecryptReader<R> {
        SyncDecryptReader::new(reader, self.data_cipher.clone())
    }

    pub fn decrypt_data(&self, data: &[u8], out: &mut Vec<u8>) -> Result<usize, std::io::Error> {
        let reader = Cursor::new(data);

        self.decrypt_reader_sync(reader).read_to_end(out)
    }
}

#[cfg(test)]
mod tests {
    use super::Cipher;

    #[test]
    fn test_encrypt_filename() {
        // tested with rclone 1.60
        let cipher = Cipher::new("testpassword", None);

        assert_eq!(
            cipher.encrypt_filename("testfilename".into()),
            "mvedi866srqc97sl5948oaej2g"
        );
        assert_eq!(
            cipher.encrypt_filename("testfilenametestfilename".into()),
            "7dpcdrb8vdgnmm0g0n5behih3pltha7hllb1mncolkrqgp3p8b2g"
        );
        assert_eq!(
            cipher.encrypt_filename("testfilenametestfilenametestfilename".into()),
            "ok5l5eohh73jdulluibldf7grqa9r4q9jekamuqkt7is8fdjsn4f32cukfrnagu35bdsc63i9pqos"
        );
        assert_eq!(
            cipher.encrypt_filename("testfilenametestfilenametestfilenametestfilename".into()),
            "hvrag3t30d3av1c31lp7j7klq95j12ru932ujf4hmaf6b4h2f2ooro1gnprne2itnibamt67h7j5a2bn1c0gkqni2n4pb17937rg22g"
        );
        assert_eq!(
            cipher.encrypt_filename("testfilenametestfilenametestfilenametestfilenametestfilename".into()),
            "013kcoiml8e3017c34132bdri58a77qea5i5f56npucna727c9tttepfe468e1aj1dmr0aqmn6rtbe2e3j8cgt7qr2rpfrpr4p12vj8"
        );
        assert_eq!(
            cipher.encrypt_filename("testfilenametestfilenametestfilenametestfilenametestfilenametestfilename".into()),
            "shcqqn35vr6ehsn5dqft11fpmr7em9krg0hvlbts0a5jb1nok9pc706c5jrjt0kug67oto6rt2o7bpqidtddvj16p2js37cgautb0856f7cnk6b4jluu3rvii0q4s87e"
        );
        assert_eq!(
            cipher.encrypt_filename("testfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilename".into()),
            "jhqr2n0782nqst11m4r5u7k6i1knb9lottugfng4m9l645ts99cr7eggld663llijgsqgrt2mqjm8b5asfb422drg3tnmthv5oj8vt0gkoura72hudp381r62si4les65hpm7jppvf0t9lgf40ime30cvs"
        );
        assert_eq!(
            cipher.encrypt_filename("testfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilename".into()),
            "5vf0sm8ao57u2k44cpsg8u40uev5e0s4jor1qk7lm351c8m4uroglehuqtlqk93kukq4r1spahkk7bnsigre5up5mlm9s4ssgl3colsn6gcqodvbluj0d2ub7jnbvqqdiutbokm6a27ko4idef590kc70t998kegekv86ij24u3p8ao9k840"
        );
        assert_eq!(
            cipher.encrypt_filename("testfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilename".into()),
            "i0eqc8m8rsb2192altkfkhh387tunp8lcm14omfbmjneuejjipncpl760rrlfakh0dikhct7i63d84gqsca6kdint24fcfp7mhjfng7hpat8s60jo328n61oborkuqjr8qp81vt5p8ogsnj95sc4p8u1kh64k3130tnicstnvsmrmgpc646g"
        );
        assert_eq!(
            cipher.encrypt_filename("testfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilename".into()),
            "05oun5jfto8bd68aceui7cgjg44hoi0m8lv3shlqu21vro8ii1o7p4b2bg3mq21qlvj7jsj4n8smrrsc8h5be2s715cu6dj1g1o7jmju62uingea7v9q1sh3dkr77v3pfd14lu1tjpfq9b8qt8dsoiq7dukhpjhv9f8se91j0p69qrkmbf1cucht23ka043v3o9jcug272gjg"
        );
        assert_eq!(
            cipher.encrypt_filename("testfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilename".into()),
            "s8a8636o3oo1g95i5eh8cg2emaodulttad0460n83btqcukgsrfuna8kjoshtdis1nkh2v30hs0sas90443d59tqofvcsi1r7pd3eoi23mivj3bf3k5vl508doaqghh40t9n74ibhe22jc5n9buq7tn7kj897tjmhd7du3vqc1s8t97drmj7tekuq9f3nb3gmsfufv53adtth9g9qkpicendfl8umro0pk4asv8"
        );
        assert_eq!(
            cipher.encrypt_filename("testfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilename".into()),
            "aih8hkkjcirtgsg067bg7g9ait5ets26p1vkctjfvmit3h6kun49cjabq01s2vrq9ia2k1q453f1uimjunf98qaja570as68irjdlhtir2vjlvfl6lmj5b18mmb3la3g8f5hhg5bf3kascg93mta0hd32blmur08deg59safibo75hvk51rtgf45r9o6v29qc4gi3kkv89k9rtlf1r4qm5ughavlqigr4ri640nhhf8b12ntidod6tv2jajaomb1"
        );
    }

    #[test]
    fn test_encrypt_path() {
        let cipher = Cipher::new("testpassword", None);

        assert_eq!(cipher.encrypt_path("/"), "/");
        assert_eq!(
            cipher.encrypt_path("/testfilename"),
            "/mvedi866srqc97sl5948oaej2g"
        );
        assert_eq!(
            cipher.encrypt_path("/testfilename/testfilename"),
            "/mvedi866srqc97sl5948oaej2g/mvedi866srqc97sl5948oaej2g"
        );
    }

    #[test]
    fn test_decrypt_filename() {
        // tested with rclone 1.60
        let cipher = Cipher::new("testpassword", None);

        assert_eq!(
            cipher
                .decrypt_filename("mvedi866srqc97sl5948oaej2g".into())
                .unwrap(),
            "testfilename"
        );
        assert_eq!(
            cipher
                .decrypt_filename("7dpcdrb8vdgnmm0g0n5behih3pltha7hllb1mncolkrqgp3p8b2g".into())
                .unwrap(),
            "testfilenametestfilename"
        );
        assert_eq!(
            cipher
                .decrypt_filename(
                    "ok5l5eohh73jdulluibldf7grqa9r4q9jekamuqkt7is8fdjsn4f32cukfrnagu35bdsc63i9pqos"
                )
                .unwrap(),
            "testfilenametestfilenametestfilename"
        );
        assert_eq!(
            cipher.decrypt_filename("hvrag3t30d3av1c31lp7j7klq95j12ru932ujf4hmaf6b4h2f2ooro1gnprne2itnibamt67h7j5a2bn1c0gkqni2n4pb17937rg22g").unwrap(),
            "testfilenametestfilenametestfilenametestfilename"
        );
        assert_eq!(
            cipher.decrypt_filename("013kcoiml8e3017c34132bdri58a77qea5i5f56npucna727c9tttepfe468e1aj1dmr0aqmn6rtbe2e3j8cgt7qr2rpfrpr4p12vj8").unwrap(),
            "testfilenametestfilenametestfilenametestfilenametestfilename"
        );
        assert_eq!(
            cipher.decrypt_filename("shcqqn35vr6ehsn5dqft11fpmr7em9krg0hvlbts0a5jb1nok9pc706c5jrjt0kug67oto6rt2o7bpqidtddvj16p2js37cgautb0856f7cnk6b4jluu3rvii0q4s87e").unwrap(),
            "testfilenametestfilenametestfilenametestfilenametestfilenametestfilename"
        );
        assert_eq!(
            cipher.decrypt_filename("jhqr2n0782nqst11m4r5u7k6i1knb9lottugfng4m9l645ts99cr7eggld663llijgsqgrt2mqjm8b5asfb422drg3tnmthv5oj8vt0gkoura72hudp381r62si4les65hpm7jppvf0t9lgf40ime30cvs").unwrap(),
            "testfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilename"
        );
        assert_eq!(
            cipher.decrypt_filename("5vf0sm8ao57u2k44cpsg8u40uev5e0s4jor1qk7lm351c8m4uroglehuqtlqk93kukq4r1spahkk7bnsigre5up5mlm9s4ssgl3colsn6gcqodvbluj0d2ub7jnbvqqdiutbokm6a27ko4idef590kc70t998kegekv86ij24u3p8ao9k840").unwrap(),
            "testfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilename"
        );
        assert_eq!(
            cipher.decrypt_filename("i0eqc8m8rsb2192altkfkhh387tunp8lcm14omfbmjneuejjipncpl760rrlfakh0dikhct7i63d84gqsca6kdint24fcfp7mhjfng7hpat8s60jo328n61oborkuqjr8qp81vt5p8ogsnj95sc4p8u1kh64k3130tnicstnvsmrmgpc646g").unwrap(),
            "testfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilename"
        );
        assert_eq!(
            cipher.decrypt_filename("05oun5jfto8bd68aceui7cgjg44hoi0m8lv3shlqu21vro8ii1o7p4b2bg3mq21qlvj7jsj4n8smrrsc8h5be2s715cu6dj1g1o7jmju62uingea7v9q1sh3dkr77v3pfd14lu1tjpfq9b8qt8dsoiq7dukhpjhv9f8se91j0p69qrkmbf1cucht23ka043v3o9jcug272gjg").unwrap(),
            "testfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilename"
        );
        assert_eq!(
            cipher.decrypt_filename("s8a8636o3oo1g95i5eh8cg2emaodulttad0460n83btqcukgsrfuna8kjoshtdis1nkh2v30hs0sas90443d59tqofvcsi1r7pd3eoi23mivj3bf3k5vl508doaqghh40t9n74ibhe22jc5n9buq7tn7kj897tjmhd7du3vqc1s8t97drmj7tekuq9f3nb3gmsfufv53adtth9g9qkpicendfl8umro0pk4asv8").unwrap(),
            "testfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilename"
        );
        assert_eq!(
            cipher.decrypt_filename("aih8hkkjcirtgsg067bg7g9ait5ets26p1vkctjfvmit3h6kun49cjabq01s2vrq9ia2k1q453f1uimjunf98qaja570as68irjdlhtir2vjlvfl6lmj5b18mmb3la3g8f5hhg5bf3kascg93mta0hd32blmur08deg59safibo75hvk51rtgf45r9o6v29qc4gi3kkv89k9rtlf1r4qm5ughavlqigr4ri640nhhf8b12ntidod6tv2jajaomb1").unwrap(),
            "testfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilenametestfilename"
        );
    }

    #[test]
    fn test_decrypt_path() {
        let cipher = Cipher::new("testpassword", None);

        assert_eq!(cipher.decrypt_path("/").unwrap(), "/");
        assert_eq!(
            cipher.decrypt_path("/mvedi866srqc97sl5948oaej2g").unwrap(),
            "/testfilename"
        );
        assert_eq!(
            cipher
                .decrypt_path("/mvedi866srqc97sl5948oaej2g/mvedi866srqc97sl5948oaej2g")
                .unwrap(),
            "/testfilename/testfilename"
        );
    }

    #[test]
    fn test_encrypt_data() {
        // tested with rclone 1.60
        let cipher = Cipher::new("testpassword", None);

        let mut encrypted = Vec::new();

        let res = cipher.encrypt_data("testdata".as_bytes(), &mut encrypted);

        assert_eq!(res.unwrap(), 56);

        let mut decrypted = Vec::new();

        let res = cipher.decrypt_data(&encrypted, &mut decrypted);

        assert_eq!(res.unwrap(), 8);

        assert_eq!(std::str::from_utf8(&decrypted).unwrap(), "testdata");
    }

    #[test]
    fn test_decrypt_data() {
        // tested with rclone 1.60
        let cipher = Cipher::new("testpassword", None);

        let encrypted = vec![
            82, 67, 76, 79, 78, 69, 0, 0, 209, 27, 246, 23, 134, 105, 131, 148, 3, 49, 228, 74,
            200, 43, 245, 170, 123, 102, 24, 137, 45, 77, 53, 115, 206, 216, 221, 4, 40, 177, 52,
            14, 5, 190, 84, 192, 246, 157, 207, 154, 11, 178, 94, 181, 135, 59, 240, 115,
        ];

        let mut decrypted = Vec::new();

        let res = cipher.decrypt_data(&encrypted, &mut decrypted);

        assert_eq!(res.unwrap(), 8);

        assert_eq!(std::str::from_utf8(&decrypted).unwrap(), "testdata");
    }
}
