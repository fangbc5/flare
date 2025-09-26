use serde::Serialize;

#[derive(Serialize)]
struct FeishuTextMessage {
    msg_type: String,
    content: serde_json::Value,
}