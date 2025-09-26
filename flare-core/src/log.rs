use tracing_subscriber::EnvFilter;

pub async fn init_logger() {
    // 初始化日志：若无 RUST_LOG 则默认 info
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"))
        )
        .init();
}