use async_trait::async_trait;
use flare_common::FlareResult;
use crate::notification::Notification;

#[async_trait]
pub trait Sender {
    async fn send(&self, notification: &Notification) -> FlareResult<()>;
}
