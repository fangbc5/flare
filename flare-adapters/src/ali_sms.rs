use flare_common::{SmsConfig, FlareError};
use reqwest::Client;
use serde::Serialize;
use async_trait::async_trait;
use flare_core::{Sender, Notification};
use flare_common::FlareResult;
use std::collections::HashMap;

/// 阿里云短信API请求参数
#[derive(Serialize)]
struct SmsRequest<'a> {
    #[serde(rename = "PhoneNumbers")]
    phone_numbers: &'a str,
    #[serde(rename = "SignName")]
    sign_name: &'a str,
    #[serde(rename = "TemplateCode")]
    template_code: &'a str,
    #[serde(rename = "TemplateParam")]
    template_param: &'a str,
    #[serde(rename = "Action")]
    action: &'a str,
    #[serde(rename = "Version")]
    version: &'a str,
    #[serde(rename = "RegionId")]
    region_id: &'a str,
    #[serde(rename = "AccessKeyId")]
    access_key_id: &'a str,
    #[serde(rename = "Signature")]
    signature: String,
    #[serde(rename = "SignatureMethod")]
    signature_method: &'a str,
    #[serde(rename = "SignatureVersion")]
    signature_version: &'a str,
    #[serde(rename = "SignatureNonce")]
    signature_nonce: String,
    #[serde(rename = "Timestamp")]
    timestamp: String,
}

/// 短信发送器
pub struct SmsSender {
    client: Client,
    config: SmsConfig,
}

impl SmsSender {
    pub fn new(config: SmsConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    async fn send_sms(&self, phone: &str, param_json: &str) -> FlareResult<()> {
        use uuid::Uuid;
        use hmac::{Hmac, Mac};
        use sha1::Sha1;
        use base64::Engine;
        
        // 生成时间戳和随机数 (ISO 8601 格式)
        let timestamp = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let nonce = Uuid::new_v4().to_string();
        
        // 构建请求参数
        let mut params = HashMap::new();
        params.insert("Action", "SendSms");
        params.insert("Version", "2017-05-25");
        params.insert("RegionId", "cn-hangzhou");
        params.insert("PhoneNumbers", phone);
        params.insert("SignName", &self.config.sign_name);
        params.insert("TemplateCode", &self.config.template_code);
        params.insert("TemplateParam", param_json);
        params.insert("AccessKeyId", &self.config.access_key_id);
        params.insert("SignatureMethod", "HMAC-SHA1");
        params.insert("SignatureVersion", "1.0");
        params.insert("SignatureNonce", &nonce);
        params.insert("Timestamp", &timestamp);
        
        // 生成签名
        let mut sorted_params: Vec<_> = params.iter().collect();
        sorted_params.sort_by_key(|(k, _)| *k);
        
        let query_string = sorted_params
            .iter()
            .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");
        
        let string_to_sign = format!("POST&{}&{}",
            urlencoding::encode("/"),
            urlencoding::encode(&query_string));
        
        let signing_key = format!("{}&", self.config.access_key_secret);
        let mut mac = Hmac::<Sha1>::new_from_slice(signing_key.as_bytes())
            .map_err(|e| FlareError::String(format!("HMAC key error: {}", e)))?;
        mac.update(string_to_sign.as_bytes());
        let signature = base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes());
        
        // 构建最终请求
        let req = SmsRequest {
            phone_numbers: phone,
            sign_name: &self.config.sign_name,
            template_code: &self.config.template_code,
            template_param: param_json,
            action: "SendSms",
            version: "2017-05-25",
            region_id: "cn-hangzhou",
            access_key_id: &self.config.access_key_id,
            signature,
            signature_method: "HMAC-SHA1",
            signature_version: "1.0",
            signature_nonce: nonce,
            timestamp,
        };

        let resp = self.client
            .post(&self.config.endpoint)
            .form(&req)
            .send()
            .await?;

        let text = resp.text().await?;
        println!("Sms response: {}", text);

        // 可在这里根据返回判断是否发送成功
        Ok(())
    }
}

#[async_trait]
impl Sender for SmsSender {
    async fn send(&self, notification: &Notification) -> FlareResult<()> {
        // 假设 Notification.to 是手机号，body 是 JSON 参数字符串
        self.send_sms(&notification.to, &notification.body).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_send_sms() {
        let sms_cfg = SmsConfig::from_env().expect("加载短信配置失败");
        let sender = SmsSender::new(sms_cfg);

        // 从环境变量读取测试手机号与模板参数
        let phone = "13849187734".to_string();
        let param = "{'code':'123456'}".to_string();

        // 调用发送
        let result = sender.send(&Notification {
            from: String::new(),
            to: phone,
            subject: String::new(),
            body: param,
            channel: flare_common::ChannelType::Sms,
        }).await;

        println!("短信发送完成！result: {:?}", result);
    }
}