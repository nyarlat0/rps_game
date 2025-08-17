use anyhow::anyhow;
use anyhow::{Context, Result};
use reqwest::{Client, ClientBuilder};
use shared::auth::Credentials;
use std::time::{Duration, Instant};

pub const BASE_URL: &str = "https://nyarlat.org";

pub fn client() -> Client
{
    ClientBuilder::new()
        .cookie_store(true)
        .redirect(reqwest::redirect::Policy::limited(10))
        .build()
        .expect("reqwest client")
}

pub async fn wait_healthy(path: &str,
                          timeout: Duration)
                          -> Result<()>
{
    let client = client();
    let start = Instant::now();
    let url = format!("{}/{}",
                      BASE_URL,
                      path.trim_start_matches('/'));
    loop {
        if start.elapsed() > timeout {
            anyhow::bail!("Service not healthy: {url}");
        }
        if let Ok(resp) = client.get(&url).send().await {
            if resp.status().is_success() {
                return Ok(());
            }
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
}

pub async fn register(creds: &Credentials) -> Result<()>
{
    let c = client();
    let url = format!("{}/api/auth/register", BASE_URL);
    let resp = c.post(url).json(creds).send().await?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(anyhow!("register failed: {} — {}",
                           status,
                           body));
    }
    Ok(())
}

pub async fn login(creds: &Credentials) -> Result<Client>
{
    let c = client();
    let url = format!("{}/api/auth/login", BASE_URL);

    let resp = c.post(&url)
                .json(creds)
                .send()
                .await
                .context("sending login request")?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(anyhow!("login failed: {} — {}",
                           status,
                           body));
    }

    Ok(c)
}

pub async fn check_me() -> Result<()>
{
    Ok(())
}
