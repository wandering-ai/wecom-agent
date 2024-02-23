mod error;
pub mod message;

use serde::{Deserialize, Serialize};
use std::error::Error as StdError;

// 获取Access Token时的返回结果
#[derive(Deserialize)]
struct AccessTokenResponse {
    errcode: i64,
    errmsg: String,
    access_token: String,
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

// 获取AccessToken的方法
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

// 承载相关方法的结构体
#[derive(Debug, Clone)]
pub struct WecomAgent {
    corpid: String,
    secret: String,
    access_token: String,
    client: reqwest::Client,
}

impl WecomAgent {
    /// 创建一个Agent。
    pub async fn new(corpid: &str, secret: &str) -> Result<Self, Box<dyn StdError>> {
        match get_access_token(corpid, secret).await {
            Ok(token) => Ok(Self {
                corpid: String::from(corpid),
                secret: String::from(secret),
                access_token: token,
                client: reqwest::Client::new(),
            }),
            Err(e) => Err(e),
        }
    }

    /// 更新access_token。
    pub async fn update_token(&mut self) -> Result<(), Box<dyn StdError + Send + Sync>> {
        self.access_token = get_access_token(&self.corpid, &self.secret).await?;
        Ok(())
    }

    /// 发送应用消息
    pub async fn send<T>(&self, msg: T) -> Result<MsgSendResponse, Box<dyn StdError + Send + Sync>>
    where
        T: Serialize,
    {
        let url = format!(
            "https://qyapi.weixin.qq.com/cgi-bin/message/send?access_token={}",
            &self.access_token
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
