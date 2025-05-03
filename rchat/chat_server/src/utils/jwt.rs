use crate::{AppError, User};
use jwt_simple::prelude::*;
use std::ops::Deref;

const JWT_DURATION: u64 = 60 * 60 * 24 * 7;
const JWT_ISSUER: &str = "rchat_server";
const JWT_AUDIENCE: &str = "rchat_web";

#[derive(Clone)]
pub struct EncodingKey(Ed25519KeyPair);

#[derive(Debug, Clone)]
pub struct DecodingKey(Ed25519PublicKey);

impl EncodingKey {
    pub fn load(pem: &str) -> Result<Self, AppError> {
        let key = Ed25519KeyPair::from_pem(pem)?;
        Ok(Self(key))
    }

    pub fn sign(&self, user: impl Into<User>) -> Result<String, AppError> {
        let claims = Claims::with_custom_claims(user.into(), Duration::from_secs(JWT_DURATION));
        let claims = claims.with_issuer(JWT_ISSUER).with_audience(JWT_AUDIENCE);
        Ok(self.0.sign(claims)?)
    }
}

impl DecodingKey {
    pub fn load(pem: &str) -> Result<Self, AppError> {
        let key = Ed25519PublicKey::from_pem(pem)?;
        Ok(Self(key))
    }

    pub fn verify(&self, token: &str) -> Result<User, AppError> {
        let mut opts = VerificationOptions::default();
        opts.allowed_issuers = Some(HashSet::from_strings(&[JWT_ISSUER]));
        opts.allowed_audiences = Some(HashSet::from_strings(&[JWT_AUDIENCE]));

        let claims = self.0.verify_token::<User>(token, Some(opts))?;
        Ok(claims.custom)
    }
}

pub fn generate_token(user: User, key: &EncodingKey) -> Result<String, AppError> {
    let claims = Claims::with_custom_claims(user, Duration::from_secs(JWT_DURATION));
    Ok(key.0.sign(claims)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn jwt_sign_verify_should_work() -> anyhow::Result<()> {
        let encoding_pem = include_str!("../../fixtures/encoding.pem");
        let decoding_pem = include_str!("../../fixtures/decoding.pem");
        let ek = EncodingKey::load(encoding_pem)?;
        let dk = DecodingKey::load(decoding_pem)?;

        let user = User::new(1, "ohmycloudy", "ohmycloudy@uk");
        let token = ek.sign(user.clone())?;
        let decoded_user: User = dk.verify(&token)?;

        assert_eq!(user, decoded_user);

        Ok(())

    }
}
