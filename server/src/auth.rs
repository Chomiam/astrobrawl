use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::process::Command;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub username: String,
    pub exp: u64,
}

#[derive(Debug, Deserialize)]
pub struct GitHubUserResponse {
    pub id: u64,
    pub login: String,
}

fn hmac_sha256(key: &[u8], data: &[u8]) -> [u8; 32] {
    let mut k = [0u8; 64];
    if key.len() > 64 {
        let mut hasher = Sha256::new();
        hasher.update(key);
        let res = hasher.finalize();
        k[..32].copy_from_slice(&res);
    } else {
        k[..key.len()].copy_from_slice(key);
    }

    let mut ipad = [0x36u8; 64];
    let mut opad = [0x5cu8; 64];
    for i in 0..64 {
        ipad[i] ^= k[i];
        opad[i] ^= k[i];
    }

    let mut inner = Sha256::new();
    inner.update(&ipad);
    inner.update(data);
    let inner_hash = inner.finalize();

    let mut outer = Sha256::new();
    outer.update(&opad);
    outer.update(&inner_hash);
    outer.finalize().into()
}

pub fn create_jwt(player_id: u64, username: &str, secret: &str) -> Result<String, String> {
    let exp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs()
        + (7 * 24 * 3600);

    let header_json = r#"{"alg":"HS256","typ":"JWT"}"#;
    let claims = Claims {
        sub: player_id.to_string(),
        username: username.to_string(),
        exp,
    };
    let claims_json = serde_json::to_string(&claims).map_err(|e| e.to_string())?;

    let header_b64 = URL_SAFE_NO_PAD.encode(header_json.as_bytes());
    let claims_b64 = URL_SAFE_NO_PAD.encode(claims_json.as_bytes());

    let payload = format!("{}.{}", header_b64, claims_b64);
    let sig = hmac_sha256(secret.as_bytes(), payload.as_bytes());
    let sig_b64 = URL_SAFE_NO_PAD.encode(&sig);

    Ok(format!("{}.{}", payload, sig_b64))
}

pub fn verify_jwt(token: &str, secret: &str) -> Result<Claims, String> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err("Invalid token format".into());
    }

    let payload = format!("{}.{}", parts[0], parts[1]);
    let expected_sig = hmac_sha256(secret.as_bytes(), payload.as_bytes());
    let sig_bytes = URL_SAFE_NO_PAD
        .decode(parts[2])
        .map_err(|_| "Invalid base64 signature")?;

    if sig_bytes != expected_sig {
        return Err("Signature mismatch".into());
    }

    let claims_bytes = URL_SAFE_NO_PAD
        .decode(parts[1])
        .map_err(|_| "Invalid base64 claims")?;
    let claims: Claims =
        serde_json::from_slice(&claims_bytes).map_err(|e| format!("JSON error: {}", e))?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs();

    if now > claims.exp {
        return Err("Token expired".into());
    }

    Ok(claims)
}

pub async fn exchange_github_code(
    client_id: &str,
    client_secret: &str,
    code: &str,
) -> Result<GitHubUserResponse, String> {
    // Perform OAuth exchange via curl (async spawn_blocking for zero-dependency reliability)
    let c_id = client_id.to_string();
    let c_secret = client_secret.to_string();
    let c_code = code.to_string();

    tokio::task::spawn_blocking(move || {
        // 1. Exchange code
        let token_output = Command::new("curl")
            .arg("-s")
            .arg("-H")
            .arg("Accept: application/json")
            .arg("-H")
            .arg("User-Agent: AstroBrawl-Server")
            .arg("-d")
            .arg(format!("client_id={}&client_secret={}&code={}", c_id, c_secret, c_code))
            .arg("https://github.com/login/oauth/access_token")
            .output()
            .map_err(|e| format!("curl error: {}", e))?;

        let token_json: serde_json::Value =
            serde_json::from_slice(&token_output.stdout).map_err(|e| format!("json error: {}", e))?;

        let access_token = token_json["access_token"]
            .as_str()
            .ok_or_else(|| format!("OAuth failed: {:?}", token_json))?;

        // 2. Fetch user profile
        let user_output = Command::new("curl")
            .arg("-s")
            .arg("-H")
            .arg(format!("Authorization: Bearer {}", access_token))
            .arg("-H")
            .arg("User-Agent: AstroBrawl-Server")
            .arg("https://api.github.com/user")
            .output()
            .map_err(|e| format!("curl user error: {}", e))?;

        let user: GitHubUserResponse =
            serde_json::from_slice(&user_output.stdout).map_err(|e| format!("json user error: {}", e))?;

        Ok(user)
    })
    .await
    .map_err(|e| e.to_string())?
}
