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
//!         let response = wecom_agent.send(msg).await;
//!     });
//! }
//! ```

mod error;
pub mod message;

use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::error::Error as StdError;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

// 企业微信鉴权凭据
#[derive(Debug)]
struct AccessToken {
    value: Option<String>,
    timestamp: SystemTime,
    lifetime: Duration,
}

impl AccessToken {
    /// 获取凭据内容
    pub fn value(&self) -> Option<&String> {
        self.value.as_ref()
    }

    /// 更新凭据
    pub fn update(&mut self, token: &str, timestamp: SystemTime, lifetime: Duration) {
        self.value = Some(token.to_owned());
        self.timestamp = timestamp;
        self.lifetime = lifetime;
    }

    /// 凭据是否已过期
    pub fn expired(&self) -> bool {
        match SystemTime::now().duration_since(self.timestamp) {
            Ok(duration) => duration >= self.lifetime,
            Err(_) => false,
        }
    }

    /// 凭据将在N秒后过期。注意，若凭据已过期，将返回false。必要时配合`expired()`使用。
    pub fn expire_in(&self, n: u64) -> bool {
        match SystemTime::now().duration_since(self.timestamp) {
            Ok(duration) => (duration - self.lifetime) < Duration::from_secs(n),
            Err(_) => false,
        }
    }

    /// 获取token上一次更新时刻
    pub fn timestamp(&self) -> SystemTime {
        self.timestamp
    }
}

impl Default for AccessToken {
    fn default() -> Self {
        Self {
            value: None,
            timestamp: UNIX_EPOCH,
            lifetime: Duration::from_secs(7200),
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
    /// 创建一个Agent。注意此过程不会自动初始化access token。
    pub fn new(corp_id: &str, secret: &str) -> Self {
        Self {
            corp_id: String::from(corp_id),
            secret: String::from(secret),
            access_token: RwLock::new(AccessToken::default()),
            client: reqwest::Client::new(),
        }
    }

    /// 更新access_token。使用`backoff_seconds`设定休止时段。若距离上次更新时间短于此时长，
    /// 将返回频繁更新错误。
    pub async fn update_token(
        &self,
        backoff_seconds: u64,
    ) -> Result<(), Box<dyn StdError + Send + Sync>> {
        // 获取token写权限
        let mut access_token = self.access_token.write().await;

        // 企业微信服务器对高频的接口调用存在风控措施。因此需要管制接口调用频率。
        let seconds_since_last_update = SystemTime::now()
            .duration_since(access_token.timestamp())?
            .as_secs();
        if seconds_since_last_update < backoff_seconds {
            return Err(Box::new(error::Error::new(
                -9,
                format!("Access token更新过于频繁。上次更新于{seconds_since_last_update}秒前。"),
            )));
        }

        // Fetch a new token
        let url = format!(
            "https://qyapi.weixin.qq.com/cgi-bin/gettoken?corpid={}&corpsecret={}",
            self.corp_id, self.secret,
        );
        let response = reqwest::get(url)
            .await?
            .json::<AccessTokenResponse>()
            .await?;
        if response.errcode != 0 {
            return Err(Box::<error::Error>::new(error::Error::new(
                response.errcode,
                response.errmsg,
            )));
        };

        // Update token with a write lock
        access_token.update(
            &response.access_token,
            SystemTime::now(),
            Duration::from_secs(response.expires_in),
        );
        Ok(())
    }

    /// 发送应用消息
    pub async fn send<T>(&self, msg: T) -> Result<MsgSendResponse, Box<dyn StdError + Send + Sync>>
    where
        T: Serialize,
    {
        // 需要更新Token?
        let token_should_update: bool = {
            let access_token = self.access_token.read().await;
            access_token.value().is_none() || access_token.expire_in(300) || access_token.expired()
        };
        if token_should_update {
            warn!("Token invalid. Updating...");
            let result = self.update_token(10).await;
            if let Err(e) = result {
                return Err(e);
            }
            info!("Token updated");
        }

        // API地址
        let url = {
            let access_token = self.access_token.read().await;
            format!(
                "https://qyapi.weixin.qq.com/cgi-bin/message/send?access_token={}",
                access_token
                    .value()
                    .expect("Access token should not be None.")
            )
        };

        // 第一次发送
        debug!("Sending [try 1]...");
        let mut response: MsgSendResponse = self
            .client
            .post(&url)
            .json(&msg)
            .send()
            .await?
            .json::<MsgSendResponse>()
            .await?;

        // 微信服务器主动弃用了当前token？
        if response.error_code() == 40014 {
            warn!("Token invalid. Updating...");
            let result = self.update_token(10).await;
            if let Err(e) = result {
                return Err(e);
            }

            // 第二次发送
            debug!("Sending [try 2]...");
            response = self
                .client
                .post(&url)
                .json(&msg)
                .send()
                .await?
                .json::<MsgSendResponse>()
                .await?;
        };

        debug!("Sending [Done]");
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
// 示例
// {
//     "errcode": 0,
//     "errmsg": "ok",
//     "access_token": "accesstoken000001",
//     "expires_in": 7200
// }
#[derive(Deserialize)]
struct AccessTokenResponse {
    errcode: i64,
    errmsg: String,
    access_token: String,
    expires_in: u64,
}
