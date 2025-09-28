use flare_common::{FeishuConfig, FeishuMessageType, FlareError, FlareResult};
use flare_core::{Notification, Sender};
use reqwest::Client;
use serde::Deserialize;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use base64::Engine;
use serde_json::json;

pub struct FeishuSender {
    client: Client,
    config: FeishuConfig,
}

impl FeishuSender {
    pub fn new(config: FeishuConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }
}

#[derive(Debug, Deserialize)]
struct FeishuIncoming<'a> {
    #[serde(default)]
    msg_type: Option<FeishuMessageType>,
    #[serde(default)]
    content: Option<serde_json::Value>,
    #[serde(default)]
    card: Option<serde_json::Value>,
    // 兼容直接传 {"text":"..."}
    #[serde(default)]
    text: Option<&'a str>,
}

#[async_trait::async_trait]
impl Sender for FeishuSender {
    async fn send(&self, notification: &Notification) -> FlareResult<()> {
        let mut url = self.config.webhook.clone();

        // 如果配置了 secret，需要附带签名
        if let Some(secret) = &self.config.secret {
            let ts = chrono::Utc::now().timestamp();
            let string_to_sign = format!("{}\n{}", ts, secret);

            let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes())
                .map_err(|e| FlareError::Config(format!("feishu secret error: {}", e)))?;
            mac.update(string_to_sign.as_bytes());
            let sign = base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes());

            let sep = if url.contains('?') { '&' } else { '?' };
            url = format!("{}{}timestamp={}&sign=\"{}\"", url, sep, ts, sign);
        }

        // 解析 notification.body 来支持多类型
        let parsed: Result<FeishuIncoming, _> = serde_json::from_str(&notification.body);
        let body_value = match parsed {
            Ok(incoming) => {
                let msg_type = incoming.msg_type.unwrap_or(FeishuMessageType::Text);
                match msg_type {
                    // 文本消息：content = { text }
                    FeishuMessageType::Text => {
                        let text = incoming
                            .content
                            .as_ref()
                            .and_then(|v| v.get("text").and_then(|x| x.as_str()))
                            .or(incoming.text)
                            .unwrap_or(notification.body.as_str());
                        json!({
                            "msg_type": "text",
                            "content": { "text": text }
                        })
                    }
                    // 富文本：content = { post: {...} }
                    FeishuMessageType::Post
                    // 图片：content = { image_key }
                    | FeishuMessageType::Image
                    // 文件/音频/视频/表情/分享/系统
                    | FeishuMessageType::File
                    | FeishuMessageType::Audio
                    | FeishuMessageType::Media
                    | FeishuMessageType::Sticker
                    | FeishuMessageType::ShareChat
                    | FeishuMessageType::ShareUser
                    | FeishuMessageType::System => {
                        let content = incoming.content.unwrap_or_else(|| json!({}));
                        let msg_type: &'static str = msg_type.into();
                        json!({
                            "msg_type": msg_type,
                            "content": content
                        })
                    }
                    // 卡片（交互）：使用 card 字段
                    FeishuMessageType::Interactive => {
                        let card = incoming.card.unwrap_or_else(|| json!({}));
                        json!({
                            "msg_type": "interactive",
                            "card": card
                        })
                    }
                    // 兜底：按文本处理（理论上不会到这）
                }
            }
            Err(_) => json!({ "msg_type": "text", "content": { "text": notification.body } }),
        };

        self.client
            .post(&url)
            .json(&body_value)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn build_sender() -> FeishuSender {
        dotenvy::dotenv().ok();
        let cfg = FeishuConfig::from_env().expect("缺少飞书配置");
        FeishuSender::new(cfg)
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
                "content": { "text": "hello from test" }
            }).to_string(),
            channel: flare_common::ChannelType::ImFeishu,
        };
        let _ = sender.send(&n).await;
    }

    #[tokio::test]
    async fn send_post() {
        let sender = build_sender();
        let n = Notification {
            from: String::new(),
            to: String::new(),
            subject: String::new(),
            body: serde_json::json!({
                "msg_type": "post",
                "content": { "post": { "zh_cn": { "title": "标题", "content": [[{"tag": "text", "text": "段落"}]] } } }
            }).to_string(),
            channel: flare_common::ChannelType::ImFeishu,
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
                "content": { "image_key": "img_xxx" }
            }).to_string(),
            channel: flare_common::ChannelType::ImFeishu,
        };
        let _ = sender.send(&n).await;
    }

    #[tokio::test]
    async fn send_interactive() {
        let sender = build_sender();
        let n = Notification {
            from: String::new(),
            to: String::new(),
            subject: String::new(),
            body: serde_json::json!({
                "msg_type": "interactive",
                "card": { "elements": [], "header": { "title": { "content": "card", "tag": "plain_text" } } }
            }).to_string(),
            channel: flare_common::ChannelType::ImFeishu,
        };
        let _ = sender.send(&n).await;
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
                "content": { "file_key": "file_xxx" }
            }).to_string(),
            channel: flare_common::ChannelType::ImFeishu,
        };
        let _ = sender.send(&n).await;
    }

    #[tokio::test]
    async fn send_audio() {
        let sender = build_sender();
        let n = Notification {
            from: String::new(),
            to: String::new(),
            subject: String::new(),
            body: serde_json::json!({
                "msg_type": "audio",
                "content": { "file_key": "audio_xxx" }
            }).to_string(),
            channel: flare_common::ChannelType::ImFeishu,
        };
        let _ = sender.send(&n).await;
    }

    #[tokio::test]
    async fn send_media() {
        let sender = build_sender();
        let n = Notification {
            from: String::new(),
            to: String::new(),
            subject: String::new(),
            body: serde_json::json!({
                "msg_type": "media",
                "content": { "file_key": "media_xxx" }
            }).to_string(),
            channel: flare_common::ChannelType::ImFeishu,
        };
        let _ = sender.send(&n).await;
    }

    #[tokio::test]
    async fn send_sticker() {
        let sender = build_sender();
        let n = Notification {
            from: String::new(),
            to: String::new(),
            subject: String::new(),
            body: serde_json::json!({
                "msg_type": "sticker",
                "content": { "file_key": "sticker_xxx" }
            }).to_string(),
            channel: flare_common::ChannelType::ImFeishu,
        };
        let _ = sender.send(&n).await;
    }

    #[tokio::test]
    async fn send_share_chat() {
        let sender = build_sender();
        let n = Notification {
            from: String::new(),
            to: String::new(),
            subject: String::new(),
            body: serde_json::json!({
                "msg_type": "share_chat",
                "content": { "chat_id": "oc_xxx" }
            }).to_string(),
            channel: flare_common::ChannelType::ImFeishu,
        };
        let _ = sender.send(&n).await;
    }

    #[tokio::test]
    async fn send_share_user() {
        let sender = build_sender();
        let n = Notification {
            from: String::new(),
            to: String::new(),
            subject: String::new(),
            body: serde_json::json!({
                "msg_type": "share_user",
                "content": { "open_id": "ou_xxx" }
            }).to_string(),
            channel: flare_common::ChannelType::ImFeishu,
        };
        let _ = sender.send(&n).await;
    }
}