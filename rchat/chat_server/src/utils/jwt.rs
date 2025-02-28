use crate::{AppError, User};
use jwt_simple::prelude::*;
use std::ops::Deref;

const JWT_DURATION: u64 = 60 * 60 * 24 * 7;

pub struct EncodingKey(Ed25519KeyPair);
pub struct DecodingKey(Ed25519PublicKey);

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
