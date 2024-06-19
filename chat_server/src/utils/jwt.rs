use jsonwebtoken::{
    get_current_timestamp, Algorithm, DecodingKey, EncodingKey, Header, Validation,
};
use serde::{Deserialize, Serialize};

use crate::error::AppError;

pub struct JwtEncodingKey {
    ek: EncodingKey,
    header: Header,
}

pub struct JwtDecodingKey {
    dk: DecodingKey,
    validation: Validation,
}

type UserId = i64;

impl JwtEncodingKey {
    pub fn load(sk: &[u8]) -> Result<Self, AppError> {
        let sk = EncodingKey::from_ed_pem(sk)?;
        let header = Header::new(Algorithm::EdDSA);
        Ok(JwtEncodingKey { ek: sk, header })
    }

    pub fn sign(&self, uid: UserId) -> Result<String, AppError> {
        let cliams = Cliams::new(uid);
        let token = jsonwebtoken::encode(&self.header, &cliams, &self.ek)?;
        Ok(token)
    }
}

impl JwtDecodingKey {
    pub fn load(pk: &[u8]) -> Result<Self, AppError> {
        let pk = DecodingKey::from_ed_pem(pk)?;
        let mut validation = Validation::new(jsonwebtoken::Algorithm::EdDSA);
        validation.set_audience(&[JWT_AUD]);
        validation.set_issuer(&[JWT_ISS]);

        Ok(JwtDecodingKey { dk: pk, validation })
    }

    #[allow(unused)]
    pub fn verify(&self, token: &str) -> Result<UserId, AppError> {
        let cliams = jsonwebtoken::decode::<Cliams>(token, &self.dk, &self.validation)?;
        Ok(cliams.claims.uid)
    }
}

#[derive(Serialize, Deserialize)]
struct Cliams {
    iss: String,
    aud: String,
    exp: u64,
    uid: i64,
}

const JWT_DURATION: u64 = 3600 * 24 * 7;
const JWT_ISS: &str = "chat";
const JWT_AUD: &str = "chat_web";

impl Cliams {
    #[inline]
    fn new(uid: UserId) -> Self {
        Self {
            iss: JWT_ISS.to_string(),
            aud: JWT_AUD.to_string(),
            exp: get_current_timestamp() + JWT_DURATION,
            uid,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EK: &[u8] = include_bytes!("../../fixtures/private.pem");
    const DK: &[u8] = include_bytes!("../../fixtures/public.pem");

    #[test]
    fn sign_and_verify() {
        let ek = JwtEncodingKey::load(EK).unwrap();
        let dk = JwtDecodingKey::load(DK).unwrap();
        let token = ek.sign(1).unwrap();
        let uid = dk.verify(&token).unwrap();
        assert_eq!(uid, 1);
    }
}
