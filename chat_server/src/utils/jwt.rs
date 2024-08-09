use jsonwebtoken::{
    get_current_timestamp, Algorithm, DecodingKey, EncodingKey, Header, Validation,
};
use serde::{Deserialize, Serialize};

use crate::models::{RowID, User};

pub struct JwtEncodingKey {
    ek: EncodingKey,
    header: Header,
}

pub struct JwtDecodingKey {
    dk: DecodingKey,
    validation: Validation,
}

#[derive(Clone, Default)]
pub struct UserCliams {
    pub uid: RowID,
    pub ws_id: RowID,
}

impl JwtEncodingKey {
    pub fn load(sk: &[u8]) -> anyhow::Result<Self> {
        let sk = EncodingKey::from_ed_pem(sk)?;
        let header = Header::new(Algorithm::EdDSA);
        Ok(JwtEncodingKey { ek: sk, header })
    }

    pub fn sign(&self, user: &UserCliams) -> anyhow::Result<String> {
        let cliams = Cliams::new(user);
        let token = jsonwebtoken::encode(&self.header, &cliams, &self.ek)?;
        Ok(token)
    }
}

impl JwtDecodingKey {
    pub fn load(pk: &[u8]) -> anyhow::Result<Self> {
        let pk = DecodingKey::from_ed_pem(pk)?;
        let mut validation = Validation::new(jsonwebtoken::Algorithm::EdDSA);
        validation.set_audience(&[JWT_AUD]);
        validation.set_issuer(&[JWT_ISS]);

        Ok(JwtDecodingKey { dk: pk, validation })
    }

    pub fn verify(&self, token: &str) -> anyhow::Result<UserCliams> {
        let cliams = jsonwebtoken::decode::<Cliams>(token, &self.dk, &self.validation)?;
        Ok(cliams.claims.into())
    }
}

#[derive(Serialize, Deserialize)]
struct Cliams {
    iss: String,
    aud: String,
    exp: u64,
    uid: RowID,
    ws_id: RowID,
}

const JWT_DURATION: u64 = 3600 * 24 * 7;
const JWT_ISS: &str = "chat";
const JWT_AUD: &str = "chat_web";

impl Cliams {
    #[inline]
    fn new(user: &UserCliams) -> Self {
        Self {
            iss: JWT_ISS.to_string(),
            aud: JWT_AUD.to_string(),
            exp: get_current_timestamp() + JWT_DURATION,
            uid: user.uid,
            ws_id: user.ws_id,
        }
    }
}

impl From<Cliams> for UserCliams {
    fn from(value: Cliams) -> Self {
        UserCliams {
            uid: value.uid,
            ws_id: value.ws_id,
        }
    }
}

impl From<User> for UserCliams {
    fn from(user: User) -> Self {
        UserCliams {
            uid: user.id,
            ws_id: user.ws_id,
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
        let token = ek.sign(&Default::default()).unwrap();
        let user = dk.verify(&token).unwrap();
        assert_eq!(user.uid, RowID::default());
    }
}
