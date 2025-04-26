use std::env;
use aes::{Aes128, Block};
use aes::cipher::{BlockDecryptMut, BlockEncryptMut, Iv, Key};
use aes::cipher::generic_array::sequence::GenericSequence;
use bytemuck::{bytes_of, from_bytes,  Pod, Zeroable};
use once_cell::sync::Lazy;
use aes::cipher::KeyIvInit;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;

pub mod generate_token;

#[derive(Pod, Zeroable, Copy, Clone, Eq, PartialEq, Debug)]
#[repr(C)]
pub struct TokenData{
    pub pid: i32,
    pub random: i32,
    pub token_id: i64
}

static AES_KEY: Lazy<Key<Aes128>> = Lazy::new(||{
    Key::<Aes128>::clone_from_slice(&hex::decode(
        env::var("ACCOUNT_AES_KEY").expect("hmac secret has not been set")
    ).expect("unable to decode ACCOUNT_AES_KEY"))
});

type Aes128CbcEnc = cbc::Encryptor<Aes128>;
type Aes128CbcDec = cbc::Decryptor<Aes128>;

impl TokenData{
    pub fn decode(token: &str) -> Option<Self>{
        let data = BASE64_STANDARD.decode(token).ok()?;

        let data: [u8; 16] = data.try_into().ok()?;

        let empty_iv = Iv::<Aes128CbcEnc>::generate(|_| 0);

        let mut aes= Aes128CbcDec::new(&*AES_KEY, &empty_iv);

        let mut block = Block::from(data);

        aes.decrypt_block_mut(&mut block);

        let data = block.as_slice();

        let token_data: &TokenData = from_bytes(data);

        Some(*token_data)
    }

    pub fn encode(&self) -> Box<str>{
        let data = bytes_of(self);
        let data: [u8; 16] = data.try_into().unwrap();

        let mut block = Block::from(data);

        let empty_iv = Iv::<Aes128CbcEnc>::generate(|_| 0);

        let mut aes= Aes128CbcEnc::new(&*AES_KEY, &empty_iv);

        aes.encrypt_block_mut(&mut block);

        let data = block.as_slice();

        BASE64_STANDARD.encode(data).into_boxed_str()
    }
}

#[cfg(test)]
mod test{
    use std::env;
    use crate::nnid::oauth::{TokenData};

    #[test]
    fn test_encode_decode(){
        unsafe{ env::set_var("ACCOUNT_AES_KEY", "0123456789abcdef0123456789abcdef"); }

        let token_data = TokenData{
            pid: 1,
            random: 2,
            token_id: 3
        };

        let enc_data = token_data.encode();

        let decrypted_token = TokenData::decode(&enc_data).unwrap();

        assert_eq!(token_data, decrypted_token)
    }
}