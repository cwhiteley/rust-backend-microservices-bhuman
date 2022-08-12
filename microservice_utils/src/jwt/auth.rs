use anyhow::Error;
use axum::extract::TypedHeader;
use headers::{authorization::Bearer, Authorization};
use chrono::Duration;
use jsonwebtoken::{
    decode, encode, errors::ErrorKind, DecodingKey, EncodingKey, Header, Validation,
};
use serde::Deserialize;
use serde::Serialize;
use std::ops::Add;

use crate::server::grpc::check_token;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub company: String,
    pub exp: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Token {
    pub access_token: String,
    pub refresh_token: String,
}

fn gen_jwt(claims: &Claims) -> String {
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret("secret".as_ref()),
    );
    token.unwrap()
}

pub fn create_token(user_id: &String) -> Token {
    let acc_claims = Claims {
        company: "hailey".to_string(),
        exp: chrono::Utc::now().add(Duration::days(7)).timestamp() as usize,
        sub: user_id.to_string(),
    };

    let ref_claims = Claims {
        company: "hailey".to_string(),
        exp: chrono::Utc::now().add(Duration::days(30)).timestamp() as usize,
        sub: user_id.to_string(),
    };

    let access_token = gen_jwt(&acc_claims);
    let refresh_token = gen_jwt(&ref_claims);
    Token {
        access_token,
        refresh_token,
    }
}

pub async fn jwt_auth(
    TypedHeader(cookies): TypedHeader<Authorization<Bearer>>,
) -> Result<String, Error> {
    let token = cookies.0.token();
    let validation = Validation::default();
    let token_data = decode::<Claims>(&token, &DecodingKey::from_secret(b"secret"), &validation)
        .map_err(|e| match *e.kind() {
            ErrorKind::InvalidToken => anyhow::anyhow!("Token is invalid"),
            ErrorKind::InvalidIssuer => anyhow::anyhow!("Issuer is invalid"),
            _ => anyhow::anyhow!("Some other errors"),
        })?;

    let user_id: &str = &token_data.claims.sub[..];
    if user_id.is_empty() {
        Err(Error::msg("User id is empty".to_string()))
    } else {
        let res = check_token(&user_id.to_string(), &token.to_string()).await;
        match res {
            Ok(_) => Ok(user_id.to_string()),
            Err(e) => Err(e),
        }
    }
}
