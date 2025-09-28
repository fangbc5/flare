use std::env;

use serde::Deserialize;
use anyhow::{Context, Result};

#[derive(Debug, Clone, Deserialize)]
pub struct EmailConfig {
    pub smtp_server: String,
    pub smtp_user: String,
    pub smtp_pass: String,
}

impl EmailConfig {
    /// 从环境变量或 `.env` 文件加载邮件配置
    pub fn from_env() -> Result<Self> {
        // 自动加载 .env 文件（如果存在）
        dotenvy::dotenv().ok();

        Ok(Self {
            smtp_server: env::var("SMTP_SERVER")
                .context("缺少 SMTP_SERVER 配置")?,
            smtp_user: env::var("SMTP_USER")
                .context("缺少 SMTP_USER 配置")?,
            smtp_pass: env::var("SMTP_PASS")
                .context("缺少 SMTP_PASS 配置")?,
        })
    }
}


#[derive(Debug, Clone, Deserialize)]
pub struct SmsConfig {
    pub endpoint: String,
    pub access_key_id: String,
    pub access_key_secret: String,
    pub sign_name: String,
    pub template_code: String,
}

impl SmsConfig {
    pub fn from_env() -> Result<Self> {
        // 自动加载 .env 文件（如果存在）
        dotenvy::dotenv().ok();

        Ok(Self {
            endpoint: env::var("SMS_ENDPOINT")
                .context("缺少 SMS_ENDPOINT 配置")?,
            access_key_id: env::var("SMS_ACCESS_KEY_ID")
                .context("缺少 SMS_ACCESS_KEY_ID 配置")?,
            access_key_secret: env::var("SMS_ACCESS_KEY_SECRET")
                .context("缺少 SMS_ACCESS_KEY_SECRET 配置")?,
            sign_name: env::var("SMS_SIGN_NAME")
                .context("缺少 SMS_SIGN_NAME 配置")?,
            template_code: env::var("SMS_TEMPLATE_CODE")
                .context("缺少 SMS_TEMPLATE_CODE 配置")?,
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct FeishuConfig {
    pub webhook: String,
    pub secret: Option<String>,
}

impl FeishuConfig {
    pub fn from_env() -> Result<Self> {
        // 自动加载 .env 文件（如果存在）
        dotenvy::dotenv().ok();

        Ok(Self {
            webhook: env::var("FEISHU_WEBHOOK").context("缺少 FEISHU_WEBHOOK 配置")?,
            secret: env::var("FEISHU_SECRET").ok(),
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct DingdingConfig {
    pub webhook: String,
    pub secret: Option<String>,
}

impl DingdingConfig {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();
        Ok(Self {
            webhook: env::var("DINGDING_WEBHOOK").context("缺少 DINGDING_WEBHOOK 配置")?,
            secret: env::var("DINGDING_SECRET").ok(),
        })
    }
}