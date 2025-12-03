//! AWS Signature Version 4 authentication

use aws_lc_rs::hmac;
use chrono::{DateTime, NaiveDateTime, Utc};
use std::collections::BTreeMap;
use thiserror::Error;

use crate::database::Database;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Missing authorization header")]
    MissingAuthHeader,
    #[error("Invalid authorization header format")]
    InvalidAuthHeader,
    #[error("Missing required header: {0}")]
    MissingHeader(String),
    #[error("Invalid date format")]
    InvalidDate,
    #[error("Access key not found")]
    AccessKeyNotFound,
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Request expired")]
    RequestExpired,
    #[error("Database error: {0}")]
    Database(#[from] crate::database::DatabaseError),
}

pub type Result<T> = std::result::Result<T, AuthError>;

#[derive(Debug, Clone)]
pub struct AuthInfo {
    pub access_key_id: String,
    pub user_name: String,
}

#[derive(Debug)]
struct ParsedAuth {
    algorithm: String,
    credential: String,
    signed_headers: Vec<String>,
    signature: String,
}

pub struct Authenticator {
    db: std::sync::Arc<Database>,
    region: String,
    service: String,
}

impl Authenticator {
    pub fn new(db: std::sync::Arc<Database>, region: &str) -> Self {
        Self {
            db,
            region: region.to_string(),
            service: "s3".to_string(),
        }
    }

    /// Authenticate a request using AWS Signature V4
    pub async fn authenticate(
        &self,
        method: &str,
        uri: &str,
        query_string: &str,
        headers: &BTreeMap<String, String>,
        payload_hash: &str,
    ) -> Result<AuthInfo> {
        // Get authorization header
        let auth_header = headers
            .get("authorization")
            .ok_or(AuthError::MissingAuthHeader)?;

        // Parse the authorization header
        let parsed = self.parse_auth_header(auth_header)?;

        // Parse credential string: AccessKeyId/date/region/service/aws4_request
        let cred_parts: Vec<&str> = parsed.credential.split('/').collect();
        if cred_parts.len() != 5 {
            return Err(AuthError::InvalidAuthHeader);
        }

        let access_key_id = cred_parts[0];
        let date_stamp = cred_parts[1];
        let region = cred_parts[2];
        let service = cred_parts[3];

        // Verify region and service
        if region != self.region || service != self.service {
            return Err(AuthError::InvalidAuthHeader);
        }

        // Get user from database
        let user = self
            .db
            .get_user_by_access_key(access_key_id)
            .await?
            .ok_or(AuthError::AccessKeyNotFound)?;

        // Get the x-amz-date header
        let amz_date = headers
            .get("x-amz-date")
            .ok_or(AuthError::MissingHeader("x-amz-date".to_string()))?;

        // Verify the date is recent (within 15 minutes)
        self.verify_date(amz_date)?;

        // Calculate the expected signature
        let string_to_sign = self.create_string_to_sign(
            method,
            uri,
            query_string,
            headers,
            &parsed.signed_headers,
            payload_hash,
            amz_date,
        )?;

        let expected_signature =
            self.calculate_signature(&user.secret_access_key, date_stamp, &string_to_sign);

        // Compare signatures
        if expected_signature != parsed.signature {
            return Err(AuthError::InvalidSignature);
        }

        Ok(AuthInfo {
            access_key_id: access_key_id.to_string(),
            user_name: user.name,
        })
    }

    fn parse_auth_header(&self, header: &str) -> Result<ParsedAuth> {
        // Format: AWS4-HMAC-SHA256 Credential=..., SignedHeaders=..., Signature=...
        let parts: Vec<&str> = header.splitn(2, ' ').collect();
        if parts.len() != 2 {
            return Err(AuthError::InvalidAuthHeader);
        }

        let algorithm = parts[0].to_string();
        if algorithm != "AWS4-HMAC-SHA256" {
            return Err(AuthError::InvalidAuthHeader);
        }

        let mut credential = String::new();
        let mut signed_headers = Vec::new();
        let mut signature = String::new();

        for part in parts[1].split(", ") {
            if let Some(value) = part.strip_prefix("Credential=") {
                credential = value.to_string();
            } else if let Some(value) = part.strip_prefix("SignedHeaders=") {
                signed_headers = value.split(';').map(|s| s.to_string()).collect();
            } else if let Some(value) = part.strip_prefix("Signature=") {
                signature = value.to_string();
            }
        }

        if credential.is_empty() || signed_headers.is_empty() || signature.is_empty() {
            return Err(AuthError::InvalidAuthHeader);
        }

        Ok(ParsedAuth {
            algorithm,
            credential,
            signed_headers,
            signature,
        })
    }

    fn verify_date(&self, amz_date: &str) -> Result<()> {
        // Parse the date (format: YYYYMMDDTHHMMSSZ)
        let date = NaiveDateTime::parse_from_str(amz_date, "%Y%m%dT%H%M%SZ")
            .map_err(|_| AuthError::InvalidDate)?;

        let request_time = DateTime::<Utc>::from_naive_utc_and_offset(date, Utc);
        let now = Utc::now();

        // Allow 15 minutes clock skew
        let diff = (now - request_time).num_seconds().abs();
        if diff > 900 {
            return Err(AuthError::RequestExpired);
        }

        Ok(())
    }

    fn create_string_to_sign(
        &self,
        method: &str,
        uri: &str,
        query_string: &str,
        headers: &BTreeMap<String, String>,
        signed_headers: &[String],
        payload_hash: &str,
        amz_date: &str,
    ) -> Result<String> {
        // Create canonical request
        let canonical_request = self.create_canonical_request(
            method,
            uri,
            query_string,
            headers,
            signed_headers,
            payload_hash,
        );

        // Create string to sign
        let date_stamp = &amz_date[..8];
        let credential_scope = format!(
            "{}/{}/{}/aws4_request",
            date_stamp, self.region, self.service
        );

        let canonical_hash = sha256_hex(canonical_request.as_bytes());

        let string_to_sign = format!(
            "AWS4-HMAC-SHA256\n{}\n{}\n{}",
            amz_date, credential_scope, canonical_hash
        );

        Ok(string_to_sign)
    }

    fn create_canonical_request(
        &self,
        method: &str,
        uri: &str,
        query_string: &str,
        headers: &BTreeMap<String, String>,
        signed_headers: &[String],
        payload_hash: &str,
    ) -> String {
        // Canonical headers
        let canonical_headers: String = signed_headers
            .iter()
            .map(|h| {
                let value = headers.get(h).map(|v| v.trim()).unwrap_or("");
                format!("{}:{}\n", h.to_lowercase(), value)
            })
            .collect();

        let signed_headers_str = signed_headers.join(";");

        // Sort query parameters
        let canonical_query = self.canonicalize_query_string(query_string);

        format!(
            "{}\n{}\n{}\n{}\n{}\n{}",
            method, uri, canonical_query, canonical_headers, signed_headers_str, payload_hash
        )
    }

    fn canonicalize_query_string(&self, query_string: &str) -> String {
        if query_string.is_empty() {
            return String::new();
        }

        let mut params: Vec<(String, String)> = query_string
            .split('&')
            .filter_map(|p| {
                let mut parts = p.splitn(2, '=');
                let key = parts.next()?;
                let value = parts.next().unwrap_or("");
                Some((key.to_string(), value.to_string()))
            })
            .collect();

        params.sort();

        params
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&")
    }

    fn calculate_signature(
        &self,
        secret_access_key: &str,
        date_stamp: &str,
        string_to_sign: &str,
    ) -> String {
        // Derive signing key
        let k_date = hmac_sha256(
            format!("AWS4{}", secret_access_key).as_bytes(),
            date_stamp.as_bytes(),
        );
        let k_region = hmac_sha256(&k_date, self.region.as_bytes());
        let k_service = hmac_sha256(&k_region, self.service.as_bytes());
        let k_signing = hmac_sha256(&k_service, b"aws4_request");

        // Calculate signature
        let signature = hmac_sha256(&k_signing, string_to_sign.as_bytes());
        hex::encode(signature)
    }
}

fn hmac_sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
    let key = hmac::Key::new(hmac::HMAC_SHA256, key);
    hmac::sign(&key, data).as_ref().to_vec()
}

fn sha256_hex(data: &[u8]) -> String {
    use aws_lc_rs::digest;
    let hash = digest::digest(&digest::SHA256, data);
    hex::encode(hash.as_ref())
}

/// Calculate SHA256 hash of data and return as hex string
pub fn sha256_hex_public(data: &[u8]) -> String {
    sha256_hex(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_hex() {
        let hash = sha256_hex(b"");
        assert_eq!(
            hash,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn test_hmac_sha256() {
        let key = b"key";
        let data = b"data";
        let result = hmac_sha256(key, data);
        assert_eq!(result.len(), 32);
    }
}
