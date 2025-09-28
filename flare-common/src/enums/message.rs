use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeishuMessageType {
    /// 文本
    Text,
    /// 富文本
    Post,
    /// 图片
    Image,
    /// 文件
    File,
    /// 语音
    Audio,
    /// 视频
    Media,
    /// 表情包
    Sticker,
    /// 卡片
    Interactive,
    /// 分享群名片
    ShareChat,
    /// 分享个人名片
    ShareUser,
    /// 系统消息
    System

}

impl From<FeishuMessageType> for &'static str {
    fn from(t: FeishuMessageType) -> Self {
        match t {
            FeishuMessageType::Text => "text",
            FeishuMessageType::Post => "post",
            FeishuMessageType::Image => "image",
            FeishuMessageType::File => "file",
            FeishuMessageType::Audio => "audio",
            FeishuMessageType::Media => "media",
            FeishuMessageType::Sticker => "sticker",
            FeishuMessageType::Interactive => "interactive",
            FeishuMessageType::ShareChat => "share_chat",
            FeishuMessageType::ShareUser => "share_user",
            FeishuMessageType::System => "system",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DingdingMessageType {
    /// 文本消息
    Text,
    /// Markdown 富文本
    Markdown,
    /// 链接消息
    Link,
    /// 交互卡片
    ActionCard,
    /// 消息卡片
    FeedCard,
    /// 图片
    Image,
    /// 文件
    File,
    /// 语音
    Audio,
    /// 视频
    Video,
}

impl From<DingdingMessageType> for &'static str {
    fn from(t: DingdingMessageType) -> Self {
        match t {
            DingdingMessageType::Text => "text",
            DingdingMessageType::Markdown => "markdown",
            DingdingMessageType::Link => "link",
            DingdingMessageType::ActionCard => "actionCard",
            DingdingMessageType::FeedCard => "feedCard",
            DingdingMessageType::Image => "image",
            DingdingMessageType::File => "file",
            DingdingMessageType::Audio => "audio",
            DingdingMessageType::Video => "video",
        }
    }
}