use crate::{AppError, User};
use jwt_simple::prelude::*;
use std::ops::Deref;

const JWT_DURATION: u64 = 60 * 60 * 24 * 7;
const JWT_ISSUER: &str = "rchat_server";
const JWT_AUDIENCE: &str = "rchat_web";

pub struct EncodingKey(Ed25519KeyPair);
pub struct DecodingKey(Ed25519PublicKey);

impl EncodingKey {
    pub fn load(pem: &str) -> Result<Self, AppError> {
        let key = Ed25519KeyPair::from_pem(pem)?;
        Ok(Self(key))
    }

    pub fn sign(user: User, key: &EncodingKey) -> Result<String, AppError> {
        let mut claims = Claims::with_custom_claims(user, Duration::from_secs(JWT_DURATION));
        let claims = claims.with_issuer(JWT_ISSUER).with_audience(JWT_AUDIENCE);
        Ok(key.sign(claims)?)
    }
}

impl DecodingKey {
    pub fn load(pem: &str) -> Result<Self, AppError> {
        let key = Ed25519PublicKey::from_pem(pem)?;
        Ok(Self(key))
    }

    pub fn verify(token: &str, key: &DecodingKey) -> Result<User, AppError> {
        let mut opts = VerificationOptions::default();
        opts.allowed_issuers = Some(HashSet::from_strings(&[JWT_ISSUER]));
        opts.allowed_audiences = Some(HashSet::from_strings(&[JWT_AUDIENCE]));

        let claims = key.verify_token::<User>(token, Some(opts))?;
        Ok(claims.custom)
    }
}

impl Deref for EncodingKey {
    type Target = Ed25519KeyPair;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for DecodingKey {
    type Target = Ed25519PublicKey;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn generate_token(user: User, key: &EncodingKey) -> Result<String, AppError> {
    let claims = Claims::with_custom_claims(user, Duration::from_secs(JWT_DURATION));
    Ok(key.sign(claims)?)
}
