use flare_common::{FeishuConfig, FeishuMessageType, FlareError, FlareResult};
use flare_core::{Notification, Sender};
use reqwest::Client;
use serde::Serialize;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use base64::Engine;

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

#[derive(Serialize)]
struct FeishuTextMsg<'a> {
    msg_type: &'a str,
    content: FeishuTextContent<'a>,
}

#[derive(Serialize)]
struct FeishuTextContent<'a> {
    text: &'a str,
}

#[async_trait::async_trait]
impl Sender for FeishuSender {
    async fn send(&self, notification: &Notification) -> FlareResult<()> {
        // 仅处理文本消息：使用 notification.body 作为文本
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

        let payload = FeishuTextMsg { msg_type: FeishuMessageType::Text.into(), content: FeishuTextContent { text: &notification.body } };

        self.client
            .post(&url)
            .json(&payload)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}