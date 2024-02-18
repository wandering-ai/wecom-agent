use reqwest;
use serde::{Deserialize, Serialize};
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
    response_code: String,
}

// 文本消息
#[derive(Debug, Serialize, PartialEq)]
pub struct TextMsgContent {
    pub content: String,
}

// 文本消息结构体
#[derive(Debug, Serialize, PartialEq)]
pub struct TextMsg {
    pub touser: String,
    pub toparty: String,
    pub totag: String,
    pub msgtype: String,
    pub agentid: usize,
    pub text: TextMsgContent,
    pub safe: i64,
    pub enable_id_trans: i64,
    pub enable_duplicate_check: i64,
    pub duplicate_check_interval: usize,
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

    /// 发送文本消息
    pub async fn send_text(&self, msg: &TextMsg) -> Result<MsgSendResponse, Box<dyn Error>> {
        if self.access_token.is_none() {
            return Err("Can not send message. Access token is None.".into());
        }
        let url = format!(
            "https://qyapi.weixin.qq.com/cgi-bin/message/send?access_token={}",
            self.access_token.as_ref().unwrap()
        );
        let response = self
            .client
            .post(&url)
            .json(msg)
            .send()
            .await?
            .json::<MsgSendResponse>()
            .await?;
        Ok(response)
    }
}
