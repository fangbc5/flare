use flare_core::init_logger;
use rdkafka::{consumer::{Consumer, StreamConsumer}, ClientConfig, Message};
use futures_util::StreamExt;
use tracing::info;
use flare_common::{EmailConfig, SmsConfig, FeishuConfig};
use flare_adapters::{EmailSender, SmsSender, FeishuSender};
use crate::handlers::HandlerContext;


mod handlers;

#[tokio::main]
async fn main() {
    // 加载环境变量并从配置构建 EmailConfig
    dotenvy::dotenv().ok();
    // 初始化日志
    init_logger().await;
    let consumer: StreamConsumer = ClientConfig::new()
        .set("group.id", "flare-workers") // 多节点消费时保证同组，消息仅消费一次
        .set("bootstrap.servers", "localhost:9092")
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "true")
        .create()
        .expect("Failed to create Kafka consumer");

    consumer
        .subscribe(&["flare-messages"])
        .expect("Can't subscribe to topic");

    // 创建发送器上下文
    let email_cfg = EmailConfig::from_env().expect("加载邮件配置失败");
    let sms_cfg = SmsConfig::from_env().expect("加载短信配置失败");
    let feishu_cfg = FeishuConfig::from_env().expect("加载飞书配置失败");
    let ctx = HandlerContext {
        email_sender: EmailSender::new(&email_cfg),
        sms_sender: SmsSender::new(sms_cfg),
        feishu_sender: FeishuSender::new(feishu_cfg),
    };

    let mut stream = consumer.stream();

    info!("Worker started, waiting for messages...");

    while let Some(result) = stream.next().await {
        match result {
            Ok(m) => {
                if let Some(payload) = m.payload_view::<str>().transpose().unwrap() {
                    info!("received message payload: {}", payload);
                    match serde_json::from_str::<crate::handlers::Message>(payload) {
                        Ok(msg) => {
                            println!("Received: {:?}", msg);
                            handlers::dispatch(&ctx, msg).await;
                        }
                        Err(e) => eprintln!("Invalid message: {}", e),
                    }
                }
            }
            Err(e) => eprintln!("Kafka error: {}", e),
        }
    }
}
