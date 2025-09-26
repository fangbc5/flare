use async_trait::async_trait;
use lettre::message::{Mailbox, Message};
use lettre::transport::smtp::AsyncSmtpTransport;
use lettre::AsyncTransport;
use flare_common::{EmailConfig, FlareResult};
use flare_core::Sender;
use flare_core::Notification;

pub struct EmailSender {
    mailer: AsyncSmtpTransport<lettre::Tokio1Executor>,
}

impl EmailSender {
    pub fn new(config: &EmailConfig) -> Self {
        let mailer = AsyncSmtpTransport::<lettre::Tokio1Executor>::relay(&config.smtp_server)
            .unwrap()
            .credentials(lettre::transport::smtp::authentication::Credentials::new(
                config.smtp_user.clone(),
                config.smtp_pass.clone(),
            ))
            .build();

        Self { mailer }
    }
}

#[async_trait]
impl Sender for EmailSender {
    async fn send(&self, notification: &Notification) -> FlareResult<()> {
        let email = Message::builder()
            .from(notification.from.parse::<Mailbox>()?)
            .to(notification.to.parse::<Mailbox>()?)
            .subject(&notification.subject)
            .body(notification.body.clone())?;

        self.mailer.send(email).await?;
        Ok(())
    }

}

#[cfg(test)]
mod tests {
    use flare_common::ChannelType;

    use super::*;

    #[tokio::test]
    async fn test_send_email() {
        let email_cfg = EmailConfig::from_env().expect("加载邮件配置失败");
        let email_sender = EmailSender::new(&email_cfg);


        let notification = Notification {
            from: email_cfg.smtp_user.clone(),
            to: "fangbaichun@beemwork.com".into(),
            subject: std::env::var("EMAIL_SUBJECT").unwrap_or_else(|_| "测试邮件".into()),
            body: std::env::var("EMAIL_BODY").unwrap_or_else(|_| "这是一封测试邮件".into()),
            channel: ChannelType::Email,
        };

        email_sender.send(&notification).await.unwrap();
        println!("邮件发送完成！");
    }
}