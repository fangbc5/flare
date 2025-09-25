flare
├── flare-api         # 对外 API 层 (REST/gRPC)
├── flare-core        # 核心业务逻辑 (通知调度/路由/模板引擎)
├── flare-adapters    # 各类适配器 (SMS, Email, IM, Push 等)
│   ├── sms-twilio
│   ├── sms-aliyun
│   ├── email-smtp
│   ├── im-slack
│   ├── im-wechat
│   └── site-message
├── flare-storage     # 存储层 (Postgres/Redis/SQLite)
├── flare-worker      # 异步任务处理 (队列消费者)
└── flare-common      # 公共模块 (配置/日志/错误/模型)
