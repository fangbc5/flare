# Flare - 统一消息通知系统

Flare 是一个基于 Rust 构建的高性能、可扩展的统一消息通知系统，支持多种消息渠道和消息类型。

## 🚀 特性

- **多渠道支持**：邮件、短信、即时通讯（飞书、钉钉）、推送通知
- **异步处理**：基于 Kafka 的异步消息队列，支持高并发
- **类型安全**：使用 Rust 的类型系统确保消息格式正确性
- **可扩展架构**：模块化设计，易于添加新的消息渠道
- **配置灵活**：支持环境变量配置，支持签名验证
- **错误处理**：完善的错误处理和日志记录

## 📁 项目结构

```
flare/
├── flare-api         # 对外 API 层 (REST/gRPC)
├── flare-core        # 核心业务逻辑 (通知调度/路由/模板引擎)
├── flare-adapters    # 各类适配器 (SMS, Email, IM, Push 等)
│   ├── email.rs      # SMTP 邮件发送
│   ├── ali_sms.rs    # 阿里云短信服务
│   ├── im_feishu.rs  # 飞书机器人
│   ├── im_dingding.rs # 钉钉机器人
│   ├── im_wechat.rs  # 企业微信 (待实现)
│   └── im_dingding.rs # 钉钉
├── flare-storage     # 存储层 (Postgres/Redis/SQLite)
├── flare-worker      # 异步任务处理 (队列消费者)
└── flare-common      # 公共模块 (配置/日志/错误/模型)
```

## 🛠️ 快速开始

### 环境要求

- Rust 1.75+
- Kafka (用于消息队列)
- SMTP 服务器 (用于邮件发送)

### 安装

```bash
git clone https://github.com/your-org/flare.git
cd flare
cargo build
```

### 配置

1. **复制环境变量示例文件**：
```bash
cp env.example .env
```

2. **编辑 `.env` 文件**，填入实际配置值：

```bash
# Kafka 配置
KAFKA_BOOTSTRAP_SERVERS=localhost:9092
KAFKA_TOPIC=flare-messages

# 邮件配置
SMTP_SERVER=smtp.example.com
SMTP_USER=your-email@example.com
SMTP_PASS=your-password

# 短信配置 (阿里云)
SMS_ENDPOINT=https://dysmsapi.aliyuncs.com
SMS_ACCESS_KEY_ID=your-access-key-id
SMS_ACCESS_KEY_SECRET=your-access-key-secret
SMS_SIGN_NAME=your-sign-name
SMS_TEMPLATE_CODE=your-template-code

# 飞书配置
FEISHU_WEBHOOK=https://open.feishu.cn/open-apis/bot/v2/hook/xxx
FEISHU_SECRET=your-secret  # 可选

# 钉钉配置
DINGDING_WEBHOOK=https://oapi.dingtalk.com/robot/send?access_token=xxx
DINGDING_SECRET=your-secret  # 可选
```

> **注意**：`env.example` 文件包含了所有必要的环境变量配置示例，请根据实际需求修改 `.env` 文件中的值。

### 运行

```bash
# 启动 Worker
cargo run -p flare-worker

# 运行测试
cargo test
```

## 📨 消息格式

### Kafka 消息格式

```json
{
  "id": "unique-message-id",
  "timestamp": "2025-01-26T10:00:00Z",
  "source": "system",
  "channel": "email|sms|im_feishu|im_dingding",
  "payload": {
    // 根据 channel 类型填充相应字段
  }
}
```

### 邮件消息

```json
{
  "channel": "email",
  "payload": {
    "to": "user@example.com",
    "subject": "邮件主题",
    "body": "邮件内容"
  }
}
```

### 短信消息

```json
{
  "channel": "sms",
  "payload": {
    "to": "13800000000",
    "param": "{\"code\":\"123456\"}"
  }
}
```

### 飞书消息

```json
{
  "channel": "im_feishu",
  "payload": {
    "text": "飞书消息内容"
  }
}
```

### 钉钉消息

```json
{
  "channel": "im_dingding",
  "payload": {
    "text": "钉钉消息内容"
  }
}
```

## 🔧 开发

### 添加新的消息渠道

1. 在 `flare-adapters/src/` 中创建新的适配器
2. 实现 `Sender` trait
3. 在 `flare-common/src/enums/channel.rs` 中添加新的 `ChannelType`
4. 在 `flare-worker/src/handlers.rs` 中添加处理逻辑

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定适配器测试
cargo test -p flare-adapters im_feishu::tests
cargo test -p flare-adapters im_dingding::tests
```

## 📄 许可证

本项目采用 [Apache 2.0 许可证](LICENSE)。

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

## 📞 支持

如有问题，请提交 Issue 或联系维护者。
