use crate::error::Error;
use serde::Serialize;
use serde_json::{json, Value};

pub trait WecomMessage {
    fn msg_type(&self) -> MessageType;
    fn key(&self) -> String;
    fn value(&self) -> impl Serialize
    where
        Self: Serialize,
    {
        self
    }
}

#[derive(Debug, Serialize)]
pub enum MessageType {
    Text,
    Image,
    Audio,
    Video,
    File,
    TextCard,
    News,
    Markdown,
}

#[derive(Debug)]
pub struct MessageBuilder {
    users: Option<String>,
    groups: Option<String>,
    tags: Option<String>,
    agent_id: Option<usize>,
    safe: i64,
    enable_id_trans: i64,
    enable_duplicate_check: i64,
    duplicate_check_interval: usize,
}

impl Default for MessageBuilder {
    fn default() -> Self {
        Self {
            users: None,
            groups: None,
            tags: None,
            agent_id: None,
            safe: 0,
            enable_id_trans: 0,
            enable_duplicate_check: 0,
            duplicate_check_interval: 1800,
        }
    }
}

impl MessageBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn to_users(mut self, users: Vec<&str>) -> Self {
        self.users = Some(
            users
                .iter()
                .fold("".to_string(), |acc, &u| format!("{acc}|{u}"))
                .trim_start_matches('|')
                .to_string(),
        );
        self
    }

    pub fn to_groups(mut self, groups: Vec<&str>) -> Self {
        self.groups = Some(
            groups
                .iter()
                .fold("".to_string(), |acc, &u| format!("{acc}|{u}"))
                .trim_start_matches('|')
                .to_string(),
        );
        self
    }

    pub fn to_tags(mut self, tags: Vec<&str>) -> Self {
        self.tags = Some(
            tags.iter()
                .fold("".to_string(), |acc, &u| format!("{acc}|{u}"))
                .trim_start_matches('|')
                .to_string(),
        );
        self
    }

    pub fn from_agent(mut self, agent_id: usize) -> Self {
        self.agent_id = Some(agent_id);
        self
    }

    pub fn with_safe(mut self, safe: i64) -> Self {
        self.safe = safe;
        self
    }
    pub fn with_enable_id_trans(mut self, enable_id_trans: i64) -> Self {
        self.enable_id_trans = enable_id_trans;
        self
    }
    pub fn with_enable_duplicate_check(mut self, enable_duplicate_check: i64) -> Self {
        self.enable_duplicate_check = enable_duplicate_check;
        self
    }
    pub fn with_duplicate_check_interval(mut self, duplicate_check_interval: usize) -> Self {
        self.duplicate_check_interval = duplicate_check_interval;
        self
    }

    pub fn build<T>(&self, content: T) -> Result<Value, Box<dyn std::error::Error>>
    where
        T: Serialize + WecomMessage,
    {
        if [&self.users, &self.groups, &self.tags]
            .iter()
            .all(|&x| x.is_none())
        {
            return Err(Box::new(Error::new(-999, "收件人不可为空".to_string())));
        }

        if self.agent_id.is_none() {
            return Err(Box::new(Error::new(-999, "AgentID不可为空".to_string())));
        }

        let empty_string = "".to_string();
        let mut j = json!({
            "touser": self.users.clone().unwrap_or(empty_string.clone()),
            "toparty": self.groups.clone().unwrap_or(empty_string.clone()),
            "totag": self.tags.clone().unwrap_or(empty_string.clone()),
            "msgtype": match content.msg_type() {
                MessageType::Audio => "voice".to_string(),
                MessageType::File => "file".to_string(),
                MessageType::Image => "image".to_string(),
                MessageType::Markdown => "markdown".to_string(),
                MessageType::News => "news".to_string(),
                MessageType::Text => "text".to_string(),
                MessageType::TextCard => "textcard".to_string(),
                MessageType::Video => "video".to_string(),
            },
            "agentid": self.agent_id.expect("AgentID should not be None"),
            "safe": self.safe,
            "enable_id_trans": self.enable_id_trans,
            "enable_duplicate_check": self.enable_duplicate_check,
            "duplicate_check_interval": self.duplicate_check_interval,});
        j.as_object_mut()
            .unwrap()
            .insert(content.key(), serde_json::to_value(content.value())?);
        Ok(j)
    }
}

// 文本消息
// 示例
// {
//    "touser" : "UserID1|UserID2|UserID3",
//    "toparty" : "PartyID1|PartyID2",
//    "totag" : "TagID1 | TagID2",
//    "msgtype" : "text",
//    "agentid" : 1,
//    "text" : {
//        "content" : "Hello from <a href=\"https://yinguobing.com\">Wondering AI</a>!"
//    },
//    "safe":0,
//    "enable_id_trans": 0,
//    "enable_duplicate_check": 0,
//    "duplicate_check_interval": 1800
// }
#[derive(Debug, Serialize, PartialEq)]
pub struct Text {
    content: String,
}

impl Text {
    pub fn new(content: String) -> Self {
        Self { content }
    }
}

impl WecomMessage for Text {
    fn msg_type(&self) -> MessageType {
        MessageType::Text
    }

    fn key(&self) -> String {
        "text".to_string()
    }
}

// 图片消息
// 示例
// {
//    "touser" : "UserID1|UserID2|UserID3",
//    "toparty" : "PartyID1|PartyID2",
//    "totag" : "TagID1 | TagID2",
//    "msgtype" : "image",
//    "agentid" : 1,
//    "image" : {
//         "media_id" : "MEDIA_ID"
//    },
//    "safe":0,
//    "enable_duplicate_check": 0,
//    "duplicate_check_interval": 1800
// }
pub struct ImageMsg {}

// 语音消息
// 示例
// {
//     "touser" : "UserID1|UserID2|UserID3",
//     "toparty" : "PartyID1|PartyID2",
//     "totag" : "TagID1 | TagID2",
//     "msgtype" : "voice",
//     "agentid" : 1,
//     "voice" : {
//          "media_id" : "MEDIA_ID"
//     },
//     "enable_duplicate_check": 0,
//     "duplicate_check_interval": 1800
// }
pub struct AudioMsg {}

// 视频消息
// 示例
// {
//     "touser" : "UserID1|UserID2|UserID3",
//     "toparty" : "PartyID1|PartyID2",
//     "totag" : "TagID1 | TagID2",
//     "msgtype" : "video",
//     "agentid" : 1,
//     "video" : {
//          "media_id" : "MEDIA_ID",
//          "title" : "Title",
//          "description" : "Description"
//     },
//     "safe":0,
//     "enable_duplicate_check": 0,
//     "duplicate_check_interval": 1800
// }
pub struct VideoMsg {}

// 文件消息
// 示例
// {
//     "touser" : "UserID1|UserID2|UserID3",
//     "toparty" : "PartyID1|PartyID2",
//     "totag" : "TagID1 | TagID2",
//     "msgtype" : "file",
//     "agentid" : 1,
//     "file" : {
//          "media_id" : "1Yv-zXfHjSjU-7LH-GwtYqDGS-zz6w22KmWAT5COgP7o"
//     },
//     "safe":0,
//     "enable_duplicate_check": 0,
//     "duplicate_check_interval": 1800
// }
pub struct FileMsg {}

// 文本卡片消息
// 示例
// {
//     "touser" : "UserID1|UserID2|UserID3",
//     "toparty" : "PartyID1 | PartyID2",
//     "totag" : "TagID1 | TagID2",
//     "msgtype" : "textcard",
//     "agentid" : 1,
//     "textcard" : {
//              "title" : "领奖通知",
//              "description" : "<div class=\"gray\">2016年9月26日</div> <div class=\"normal\">恭喜你抽中iPhone 7一台，领奖码：xxxx</div><div class=\"highlight\">请于2016年10月10日前联系行政同事领取</div>",
//              "url" : "URL",
//                          "btntxt":"更多"
//     },
//     "enable_id_trans": 0,
//     "enable_duplicate_check": 0,
//     "duplicate_check_interval": 1800
// }
pub struct TextCardMsg {}

// MarkDown消息
// 示例
// {
//     "touser" : "UserID1|UserID2|UserID3",
//     "toparty" : "PartyID1|PartyID2",
//     "totag" : "TagID1 | TagID2",
//     "msgtype": "markdown",
//     "agentid" : 1,
//     "markdown": {
//          "content": "您的会议室已经预定，稍后会同步到`邮箱`  \n>**事项详情**  \n>事　项：<font color=\"info\">开会</font>  \n>组织者：@miglioguan  \n>参与者：@miglioguan、@kunliu、@jamdeezhou、@kanexiong、@kisonwang  \n>  \n>会议室：<font color=\"info\">广州TIT 1楼 301</font>  \n>日　期：<font color=\"warning\">2018年5月18日</font>  \n>时　间：<font color=\"comment\">上午9:00-11:00</font>  \n>  \n>请准时参加会议。  \n>  \n>如需修改会议信息，请点击：[修改会议信息](https://work.weixin.qq.com)"
//     },
//     "enable_duplicate_check": 0,
//     "duplicate_check_interval": 1800
// }
pub struct MarkDownMsg {}

#[cfg(test)]
mod test {
    use std::vec;

    use super::*;
    #[test]
    fn test_builder() {
        let content = Text::new("hello text!".to_string());
        let msg = MessageBuilder::default()
            .to_users(vec!["robin", "tom", "Alex", "Susanna"])
            .to_groups(vec!["a", "b", "c"])
            .to_tags(vec!["x", "y", "z"])
            .from_agent(1)
            .with_safe(1)
            .with_enable_id_trans(1)
            .with_enable_duplicate_check(1)
            .with_duplicate_check_interval(800)
            .build(content)
            .expect("Massage should be built");
        let raw = json!({
            "touser" : "robin|tom|Alex|Susanna",
            "toparty" : "a|b|c",
            "totag" : "x|y|z",
            "msgtype": "text",
            "agentid" : 1,
            "safe": 1,
            "enable_id_trans": 1,
            "enable_duplicate_check": 1,
            "duplicate_check_interval": 800,
            "text": {
                 "content": "hello text!"
            },
        });
        assert_eq!(msg, serde_json::to_value(raw).unwrap());
    }
}
