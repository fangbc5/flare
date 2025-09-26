use flare_common::ChannelType;

#[derive(Debug, Clone)]
pub struct Notification {
    pub from: String,
    pub to: String,
    pub subject: String,
    pub body: String,
    pub channel: ChannelType, // 邮件、短信、IM等
}
