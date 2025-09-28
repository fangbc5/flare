use flare_common::{ChannelType, EmailConfig, FlareError, FlareResult};
use flare_core::Notification;
use flare_core::Sender;
use flare_adapters::{EmailSender, SmsSender, FeishuSender, DingdingSender};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub timestamp: String,
    pub source: String,
    pub channel: ChannelType,
    pub payload: serde_json::Value, // 动态字段
}

pub struct HandlerContext {
    pub email_sender: EmailSender,
    pub sms_sender: SmsSender,
    pub feishu_sender: FeishuSender,
    pub dingding_sender: DingdingSender,
}

pub async fn dispatch(ctx: &HandlerContext, msg: Message) {
    let result = match msg.channel {
        ChannelType::Email => handle_email(ctx, msg).await,
        ChannelType::Sms => handle_sms(ctx, msg).await,
        ChannelType::ImFeishu => handle_im_feishu(ctx, msg).await,
        ChannelType::ImDingding => handle_im_dingding(ctx, msg).await,
        _ => Err(FlareError::Config("Unsupported message type".into())),
    };

    if let Err(e) = result {
        eprintln!("Dispatch error: {}", e);
    }
}

async fn handle_email(ctx: &HandlerContext, msg: Message) -> FlareResult<()> {
    // 从环境加载 Email 配置仅用于发件人默认值
    let EmailConfig { smtp_user, .. } = EmailConfig::from_env()
        .map_err(|e| FlareError::Config(format!("email config error: {}", e)))?;

    let from = smtp_user;
    let to = require_str(&msg.payload, "to")?;
    let subject = require_str(&msg.payload, "subject")?;
    let body = require_str(&msg.payload, "body")?;

    let notification = Notification {
        from,
        to,
        subject,
        body,
        channel: ChannelType::Email,
    };

    ctx.email_sender.send(&notification).await
}

async fn handle_sms(ctx: &HandlerContext, msg: Message) -> FlareResult<()> {
    // to: 手机号, body: 模板参数JSON字符串
    let to = require_str(&msg.payload, "to")?;
    let param = require_str(&msg.payload, "param").or_else(|_| require_str(&msg.payload, "body"))?;

    let notification = Notification {
        from: String::new(),
        to,
        subject: String::new(),
        body: param,
        channel: ChannelType::Sms,
    };

    ctx.sms_sender.send(&notification).await
}

async fn handle_im_feishu(ctx: &HandlerContext, msg: Message) -> FlareResult<()> {
    // 支持 payload.text 或 payload.body 作为文本
    let text = require_str(&msg.payload, "text")
        .or_else(|_| require_str(&msg.payload, "body"))?;

    let notification = Notification {
        from: String::new(),
        to: String::new(),
        subject: String::new(),
        body: text,
        channel: ChannelType::ImFeishu,
    };

    ctx.feishu_sender.send(&notification).await
}

async fn handle_im_dingding(ctx: &HandlerContext, msg: Message) -> FlareResult<()> {
    // 支持 payload.text 或 payload.body 作为消息内容
    let content = require_str(&msg.payload, "text")
        .or_else(|_| require_str(&msg.payload, "body"))?;

    let notification = Notification {
        from: String::new(),
        to: String::new(),
        subject: String::new(),
        body: content,
        channel: ChannelType::ImDingding,
    };

    ctx.dingding_sender.send(&notification).await
}

fn require_str(payload: &serde_json::Value, key: &str) -> FlareResult<String> {
    payload
        .get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| FlareError::Config(format!("missing or invalid '{}'", key)))
}