use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use base64::Engine;

use flare_common::{FlareError, FlareResult, DingdingConfig, DingdingMessageType};
use flare_core::{Notification, Sender};

pub struct DingdingSender {
    client: Client,
    config: DingdingConfig,
}

impl DingdingSender {
    pub fn new(config: DingdingConfig) -> Self {
        Self { client: Client::new(), config }
    }
}

#[derive(Debug, Deserialize)]
struct DingdingIncoming<'a> {
    #[serde(default)]
    msg_type: Option<DingdingMessageType>,
    #[serde(default)]
    content: Option<serde_json::Value>,
    #[serde(default)]
    text: Option<&'a str>,
}

#[async_trait::async_trait]
impl Sender for DingdingSender {
    async fn send(&self, notification: &Notification) -> FlareResult<()> {
        let mut url = self.config.webhook.clone();

        if let Some(secret) = &self.config.secret {
            let ts = chrono::Utc::now().timestamp_millis();
            let string_to_sign = format!("{}\n{}", ts, secret);
            let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes())
                .map_err(|e| FlareError::Config(format!("dingding secret error: {}", e)))?;
            mac.update(string_to_sign.as_bytes());
            let sign = base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes());
            let sep = if url.contains('?') { '&' } else { '?' };
            url = format!("{}{}timestamp={}&sign={}", url, sep, ts, urlencoding::encode(&sign));
        }

        // 解析 body：允许直接传 text，或 content 对象
        let parsed: Result<DingdingIncoming, _> = serde_json::from_str(&notification.body);
        let body_value = match parsed {
            Ok(incoming) => {
                let msg_type = incoming.msg_type.unwrap_or(DingdingMessageType::Text);
                match msg_type {
                    DingdingMessageType::Text => {
                        let text = incoming
                            .content
                            .as_ref()
                            .and_then(|v| v.get("content").and_then(|x| x.as_str()))
                            .or(incoming.text)
                            .unwrap_or(notification.body.as_str());
                        json!({ "msgtype": "text", "text": { "content": text } })
                    }
                    DingdingMessageType::Markdown => {
                        let content = incoming.content.unwrap_or_else(|| json!({}));
                        json!({ "msgtype": "markdown", "markdown": content })
                    }
                    DingdingMessageType::Link => {
                        let content = incoming.content.unwrap_or_else(|| json!({}));
                        json!({ "msgtype": "link", "link": content })
                    }
                    DingdingMessageType::ActionCard => {
                        let content = incoming.content.unwrap_or_else(|| json!({}));
                        json!({ "msgtype": "actionCard", "actionCard": content })
                    }
                    DingdingMessageType::FeedCard => {
                        let content = incoming.content.unwrap_or_else(|| json!({}));
                        json!({ "msgtype": "feedCard", "feedCard": content })
                    }
                    DingdingMessageType::Image => {
                        let content = incoming.content.unwrap_or_else(|| json!({}));
                        json!({ "msgtype": "image", "image": content })
                    }
                    DingdingMessageType::File => {
                        let content = incoming.content.unwrap_or_else(|| json!({}));
                        json!({ "msgtype": "file", "file": content })
                    }
                    DingdingMessageType::Audio => {
                        let content = incoming.content.unwrap_or_else(|| json!({}));
                        json!({ "msgtype": "audio", "audio": content })
                    }
                    DingdingMessageType::Video => {
                        let content = incoming.content.unwrap_or_else(|| json!({}));
                        json!({ "msgtype": "video", "video": content })
                    }
                }
            }
            Err(_) => json!({ "msgtype": "text", "text": { "content": notification.body } }),
        };

        println!("请求URL: {}", url);
        println!("请求体: {}", serde_json::to_string_pretty(&body_value).unwrap_or_default());
        
        let response = self.client
            .post(&url)
            .json(&body_value)
            .send()
            .await?;
            
        let status = response.status();
        let response_text = response.text().await?;
        
        println!("响应状态: {}", status);
        println!("响应内容: {}", response_text);
        
        if !status.is_success() {
            return Err(FlareError::String(format!("钉钉API错误 {}: {}", status, response_text)));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_sender() -> DingdingSender {
        dotenvy::dotenv().ok();
        let cfg = DingdingConfig::from_env().expect("缺少钉钉配置");
        DingdingSender::new(cfg)
    }

    #[tokio::test]
    async fn send_text() {
        let sender = build_sender();
        let n = Notification {
            from: String::new(),
            to: String::new(),
            subject: String::new(),
            body: serde_json::json!({
                "msg_type": "text",
                "content": { "content": "hello from dingding test" }
            }).to_string(),
            channel: flare_common::ChannelType::ImDingding,
        };
        
        println!("发送消息体: {}", n.body);
        match sender.send(&n).await {
            Ok(_) => println!("✅ 发送成功"),
            Err(e) => println!("❌ 发送失败: {}", e),
        }
    }

    #[tokio::test]
    async fn send_text_simple() {
        let sender = build_sender();
        // 测试最简单的文本消息（不解析JSON，直接作为文本发送）
        let n = Notification {
            from: String::new(),
            to: String::new(),
            subject: String::new(),
            body: "简单测试消息".to_string(),
            channel: flare_common::ChannelType::ImDingding,
        };
        
        println!("发送简单消息: {}", n.body);
        match sender.send(&n).await {
            Ok(_) => println!("✅ 简单消息发送成功"),
            Err(e) => println!("❌ 简单消息发送失败: {}", e),
        }
    }

    #[tokio::test]
    async fn send_markdown() {
        let sender = build_sender();
        let n = Notification {
            from: String::new(),
            to: String::new(),
            subject: String::new(),
            body: serde_json::json!({
                "msg_type": "markdown",
                "content": {
                    "title": "测试标题",
                    "text": "## 测试内容\n\n**粗体文本**\n\n- 列表项1\n- 列表项2"
                }
            }).to_string(),
            channel: flare_common::ChannelType::ImDingding,
        };
        let _ = sender.send(&n).await;
    }

    #[tokio::test]
    async fn send_link() {
        let sender = build_sender();
        let n = Notification {
            from: String::new(),
            to: String::new(),
            subject: String::new(),
            body: serde_json::json!({
                "msg_type": "link",
                "content": {
                    "text": "这是一个链接消息",
                    "title": "链接标题",
                    "picUrl": "https://example.com/image.jpg",
                    "messageUrl": "https://example.com"
                }
            }).to_string(),
            channel: flare_common::ChannelType::ImDingding,
        };
        let _ = sender.send(&n).await;
    }

    #[tokio::test]
    async fn send_action_card() {
        let sender = build_sender();
        let n = Notification {
            from: String::new(),
            to: String::new(),
            subject: String::new(),
            body: serde_json::json!({
                "msg_type": "action_card",
                "content": {
                    "title": "卡片标题",
                    "text": "卡片内容",
                    "singleTitle": "查看详情",
                    "singleURL": "https://example.com"
                }
            }).to_string(),
            channel: flare_common::ChannelType::ImDingding,
        };
        let _ = sender.send(&n).await;
    }

    #[tokio::test]
    async fn send_feed_card() {
        let sender = build_sender();
        let n = Notification {
            from: String::new(),
            to: String::new(),
            subject: String::new(),
            body: serde_json::json!({
                "msg_type": "feed_card",
                "content": {
                    "links": [
                        {
                            "title": "链接1",
                            "messageURL": "https://example.com/1",
                            "picURL": "https://example.com/pic1.jpg"
                        },
                        {
                            "title": "链接2", 
                            "messageURL": "https://example.com/2",
                            "picURL": "https://example.com/pic2.jpg"
                        }
                    ]
                }
            }).to_string(),
            channel: flare_common::ChannelType::ImDingding,
        };
        let _ = sender.send(&n).await;
    }

    #[tokio::test]
    async fn send_image() {
        let sender = build_sender();
        let n = Notification {
            from: String::new(),
            to: String::new(),
            subject: String::new(),
            body: serde_json::json!({
                "msg_type": "image",
                "content": {
                    "picURL": "https://inews.gtimg.com/news_bt/OBkbmPLeWLy4IM4oUDGvOIqSDSZ9lYOtW3qSXCYh78KXcAA/1000"
                }
            }).to_string(),
            channel: flare_common::ChannelType::ImDingding,
        };
        
        println!("发送图片消息体: {}", n.body);
        match sender.send(&n).await {
            Ok(_) => println!("✅ 图片消息发送成功"),
            Err(e) => println!("❌ 图片消息发送失败: {}", e),
        }
    }

    #[tokio::test]
    async fn send_file() {
        let sender = build_sender();
        let n = Notification {
            from: String::new(),
            to: String::new(),
            subject: String::new(),
            body: serde_json::json!({
                "msg_type": "file",
                "content": {
                    "mediaId": "NDoBb60VLybv3eZQuxp1jw4MJlemrZQ3",
                    "fileType": "jpeg",
                    "fileName": "20250916-172734.jpeg"
                }
            }).to_string(),
            channel: flare_common::ChannelType::ImDingding,
        };
        
        println!("发送文件消息体: {}", n.body);
        match sender.send(&n).await {
            Ok(_) => println!("✅ 文件消息发送成功"),
            Err(e) => println!("❌ 文件消息发送失败: {}", e),
        }
    }
}


