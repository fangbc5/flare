use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ChannelType {
    Email,
    Sms,
    ImFeishu,
    ImDingding,
    ImWechat,
    Push,
    SiteMessage,
}