# wecom-agent
`wecom-agent`是对企业微信API调用的轻封装，简化了各类信息的发送过程。

## 使用方法
```rust
use wecom_agent::{
    message::{MessageBuilder, Text},
    MsgSendResponse, WecomAgent,
};

async fn example() {
    let content = Text::new("Hello from Wandering AI!".to_string());
    let msg = MessageBuilder::default()
        .to_users(vec!["robin", "tom"])
        .from_agent(42)
        .build(content)
        .expect("Massage should be built");
    let handle = tokio::spawn(async move {
        let wecom_agent = WecomAgent::new("your_corpid", "your_secret")
            .await
            .expect("wecom agent should be initialized.");
        let response = wecom_agent.send(msg).await;
    });
}
```