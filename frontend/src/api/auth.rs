use gloo_net::http::Request;
use shared::{Credentials, UserInfo};

pub async fn fetch_user_info() -> Option<UserInfo>
{
    let response =
        Request::get("/api/auth/me").send().await;

    match response {
        Ok(resp) => resp.json::<UserInfo>().await.ok(),
        Err(_) => None,
    }
}

pub async fn register_user(creds: &Credentials)
                           -> Result<String, String>
{
    let response =
        Request::post("/api/auth/register").json(creds)
                                           .unwrap()
                                           .send()
                                           .await;

    match response {
        Ok(resp) => {
            let msg = resp.text().await.unwrap_or_default();
            if resp.ok() {
                Ok(msg)
            } else {
                Err(msg)
            }
        }

        Err(e) => {
            let msg = format!("Network error: {e}");
            Err(msg)
        }
    }
}

pub async fn login_user(creds: &Credentials)
                        -> Result<String, String>
{
    let response =
        Request::post("/api/auth/login").json(creds)
                                        .unwrap()
                                        .send()
                                        .await;

    match response {
        Ok(resp) => {
            let msg = resp.text().await.unwrap_or_default();
            if resp.ok() {
                Ok(msg)
            } else {
                Err(msg)
            }
        }

        Err(e) => {
            let msg = format!("Network error: {e}");
            Err(msg)
        }
    }
}
