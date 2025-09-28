use flare_core::init_logger;
use rdkafka::{consumer::{Consumer, StreamConsumer}, ClientConfig, Message};
use futures_util::StreamExt;
use tracing::info;
use flare_common::{EmailConfig, SmsConfig, FeishuConfig, DingdingConfig};
use flare_adapters::{EmailSender, SmsSender, FeishuSender, DingdingSender};
use crate::handlers::HandlerContext;


mod handlers;

#[tokio::main]
async fn main() {
    // 加载环境变量并从配置构建 EmailConfig
    dotenvy::dotenv().ok();
    // 初始化日志
    init_logger().await;
    // 从环境变量获取 Kafka 配置
    let kafka_servers = std::env::var("KAFKA_BOOTSTRAP_SERVERS")
        .unwrap_or_else(|_| "localhost:9092".to_string());
    let kafka_topic = std::env::var("KAFKA_TOPIC")
        .unwrap_or_else(|_| "flare-messages".to_string());
    
    info!("Kafka servers: {}", kafka_servers);
    info!("Kafka topic: {}", kafka_topic);

    let consumer: StreamConsumer = ClientConfig::new()
        .set("group.id", "flare-workers") // 多节点消费时保证同组，消息仅消费一次
        .set("bootstrap.servers", &kafka_servers)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "true")
        .create()
        .expect("Failed to create Kafka consumer");

    consumer
        .subscribe(&[&kafka_topic])
        .expect("Can't subscribe to topic");

    // 创建发送器上下文
    let email_cfg = EmailConfig::from_env().expect("加载邮件配置失败");
    let sms_cfg = SmsConfig::from_env().expect("加载短信配置失败");
    let feishu_cfg = FeishuConfig::from_env().expect("加载飞书配置失败");
    let dingding_cfg = DingdingConfig::from_env().expect("加载钉钉配置失败");
    let ctx = HandlerContext {
        email_sender: EmailSender::new(&email_cfg),
        sms_sender: SmsSender::new(sms_cfg),
        feishu_sender: FeishuSender::new(feishu_cfg),
        dingding_sender: DingdingSender::new(dingding_cfg),
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
