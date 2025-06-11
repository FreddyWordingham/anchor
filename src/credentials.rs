use aws_config::{BehaviorVersion, load_defaults};
use aws_sdk_ecr::Client as EcrClient;
use base64::{Engine, engine::general_purpose};
use bollard::auth::DockerCredentials;
use std::error::Error;

/// Get Docker credentials for AWS ECR (Elastic Container Registry).
pub async fn get_ecr_credentials() -> Result<DockerCredentials, Box<dyn Error>> {
    // 1. Load AWS config from environment (reads AWS_ACCESS_KEY_ID, etc.)
    let config = load_defaults(BehaviorVersion::latest()).await;
    let client = EcrClient::new(&config);

    // 2. Call ECR to get an authorization token
    let resp = client
        .get_authorization_token()
        .send()
        .await
        .map_err(|err: _| format!("ECR GetAuthToken failed: {}", err))?;

    // 3. Extract the first (and usually only) auth data
    let auth_data = resp
        .authorization_data
        .ok_or("No authorization_data returned")?
        .into_iter()
        .next()
        .ok_or("authorization_data was empty")?;

    // 4. The token is base64("username:password"), typically "AWS:<long-password>"
    let token_b64 = auth_data.authorization_token.ok_or("authorization_token missing")?;

    let decoded = general_purpose::STANDARD.decode(token_b64)?;
    let decoded_str = String::from_utf8(decoded)?;
    let mut parts = decoded_str.splitn(2, ':');
    let username = parts.next().ok_or("No username in token")?;
    let password = parts.next().ok_or("No password in token")?;

    // 5. Server address is the proxy endpoint, e.g. "123456789012.dkr.ecr.us-west-2.amazonaws.com"
    let registry = auth_data.proxy_endpoint.ok_or("proxy_endpoint missing")?;

    Ok(DockerCredentials {
        username: Some(username.to_string()),
        password: Some(password.to_string()),
        serveraddress: Some(registry),
        ..Default::default()
    })
}
