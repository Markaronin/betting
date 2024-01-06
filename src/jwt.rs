use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    user_id: String,
}

const ALGORITHM: Algorithm = Algorithm::HS256;

pub fn create_jwt(user_id: &str, secret: &str) -> String {
    let claims = Claims {
        user_id: user_id.to_string(),
    };
    encode(
        &Header::new(ALGORITHM),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .unwrap()
}

pub fn validate_and_extract_user_id(
    token: &str,
    secret: &str,
) -> Result<String, jsonwebtoken::errors::Error> {
    let mut validation = Validation::new(ALGORITHM);
    validation.validate_exp = false;
    validation.required_spec_claims.remove("exp");

    let decoded_token = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )?;

    Ok(decoded_token.claims.user_id)
}
