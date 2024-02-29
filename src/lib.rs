//! # wecom-agent
//!
//! `wecom-agent`封装了企业微信API的消息发送功能。
//!
//! ## 使用方法
//! ```rust
//! use wecom_agent::{
//!     message::{MessageBuilder, Text},
//!     MsgSendResponse, WecomAgent,
//! };
//! async fn example() {
//!     let content = Text::new("Hello from Wandering AI!".to_string());
//!     let msg = MessageBuilder::default()
//!         .to_users(vec!["robin", "tom"])
//!         .from_agent(42)
//!         .build(content)
//!         .expect("Massage should be built");
//!     let handle = tokio::spawn(async move {
//!         let wecom_agent = WecomAgent::new("your_corpid", "your_secret");
//!         wecom_agent.update_token(5).await;
//!         let response = wecom_agent.send(msg).await;
//!     });
//! }
//! ```

mod error;
pub mod message;

use serde::{Deserialize, Serialize};
use std::error::Error as StdError;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

// 企业微信鉴权凭据
#[derive(Debug)]
struct AccessToken {
    value: Option<String>,
    timestamp: SystemTime,
}

impl AccessToken {
    fn value(&self) -> Option<&String> {
        self.value.as_ref()
    }

    fn set_value(&mut self, value: Option<String>) {
        self.value = value;
    }

    fn timestamp(&self) -> SystemTime {
        self.timestamp
    }

    fn set_timestamp(&mut self, timestamp: SystemTime) {
        self.timestamp = timestamp;
    }
}

impl Default for AccessToken {
    fn default() -> Self {
        Self {
            value: None,
            timestamp: UNIX_EPOCH,
        }
    }
}

/// 企业微信API的轻量封装
#[derive(Debug)]
pub struct WecomAgent {
    corp_id: String,
    secret: String,
    access_token: RwLock<AccessToken>,
    client: reqwest::Client,
}

impl WecomAgent {
    /// 创建一个Agent。注意此过程不会不会自动初始化access token。
    pub fn new(corp_id: &str, secret: &str) -> Self {
        Self {
            corp_id: String::from(corp_id),
            secret: String::from(secret),
            access_token: RwLock::new(AccessToken::default()),
            client: reqwest::Client::new(),
        }
    }

    /// 更新access_token。
    pub async fn update_token(
        &self,
        backoff_seconds: u64,
    ) -> Result<(), Box<dyn StdError + Send + Sync>> {
        // 企业微信服务器对高频的接口调用存在风控措施。因此需要管制接口调用频率。
        let seconds_since_last_update: u64;
        {
            let access_token = self.access_token.read().await;
            seconds_since_last_update = SystemTime::now()
                .duration_since(access_token.timestamp())?
                .as_secs();
        }
        if seconds_since_last_update < backoff_seconds {
            return Err(Box::new(error::Error::new(
                -9,
                format!("Access token更新过于频繁。上次更新于{seconds_since_last_update}秒前。"),
            )));
        }

        // Fetch a new token
        let new_token = get_access_token(&self.corp_id, &self.secret).await?;

        // Update token with a write lock
        let mut access_token = self.access_token.write().await;
        access_token.set_value(Some(new_token));
        access_token.set_timestamp(SystemTime::now());
        Ok(())
    }

    /// 发送应用消息
    pub async fn send<T>(&self, msg: T) -> Result<MsgSendResponse, Box<dyn StdError + Send + Sync>>
    where
        T: Serialize,
    {
        // Safety first, is the token valid?
        let access_token = self.access_token.read().await;

        if access_token.value().is_none() {
            return Err(Box::new(error::Error::new(
                -9,
                "Access token尚未初始化。".to_owned(),
            )));
        }
        let url = format!(
            "https://qyapi.weixin.qq.com/cgi-bin/message/send?access_token={}",
            access_token
                .value()
                .expect("Access token should not be None.")
        );
        let response = self
            .client
            .post(&url)
            .json(&msg)
            .send()
            .await?
            .json::<MsgSendResponse>()
            .await?;
        Ok(response)
    }
}

// 应用消息发送结果
#[derive(Deserialize)]
pub struct MsgSendResponse {
    errcode: i64,
    errmsg: String,
    invaliduser: Option<String>,
    invalidparty: Option<String>,
    invalidtag: Option<String>,
    unlicenseduser: Option<String>,
    msgid: String,
    response_code: Option<String>,
}

impl MsgSendResponse {
    pub fn is_error(&self) -> bool {
        self.errcode != 0
    }

    pub fn error_code(&self) -> i64 {
        self.errcode
    }

    pub fn error_msg(&self) -> &str {
        &self.errmsg
    }
}

// 获取Access Token时的返回结果
#[derive(Deserialize)]
struct AccessTokenResponse {
    errcode: i64,
    errmsg: String,
    access_token: String,
}

// 获取AccessToken
async fn get_access_token(
    corpid: &str,
    secret: &str,
) -> Result<String, Box<dyn StdError + Send + Sync>> {
    let url = format!(
        "https://qyapi.weixin.qq.com/cgi-bin/gettoken?corpid={corpid}&corpsecret={secret}",
    );
    let response = reqwest::get(url)
        .await?
        .json::<AccessTokenResponse>()
        .await?;
    match response.errcode {
        0 => Ok(response.access_token),
        _ => Err(Box::<error::Error>::new(error::Error::new(
            response.errcode,
            response.errmsg,
        ))),
    }
}
