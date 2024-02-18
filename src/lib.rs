use reqwest;
use serde::Deserialize;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct WecomAgent {
    corpid: String,
    secret: String,
    access_token: Option<String>,
    client: reqwest::Client,
}

#[derive(Deserialize)]
struct AccessTokenResponse {
    errcode: i64,
    errmsg: String,
    access_token: String,
    expires_in: usize,
}

impl WecomAgent {
    /// 创建一个Agent。注意默认access_token为空，需要更新后使用。
    pub fn new(corpid: &str, secret: &str) -> Self {
        Self {
            corpid: String::from(corpid),
            secret: String::from(secret),
            access_token: None,
            client: reqwest::Client::new(),
        }
    }

    /// 检查access_token是否初始化。
    pub fn token_is_some(&self) -> bool {
        self.access_token.is_some()
    }

    /// 更新access_token。
    pub async fn update_token(&mut self) -> Result<(), Box<dyn Error>> {
        let url = format!(
            "https://qyapi.weixin.qq.com/cgi-bin/gettoken?corpid={}&corpsecret={}",
            &self.corpid, &self.secret
        );
        let response = self
            .client
            .get(url)
            .send()
            .await?
            .json::<AccessTokenResponse>()
            .await?;
        if response.errcode != 0 {
            return Err(format!(
                "Failed to fetch access token. Error code: {}, {}",
                response.errcode, response.errmsg,
            )
            .into());
        }
        self.access_token = Some(response.access_token);
        Ok(())
    }
}
