# 错误处理

- [anyhow](https://docs.rs/anyhow/latest/anyhow/index.html): 统一, 简单的错误处理, 适用于应用程序级别。
- [thiserror](https://docs.rs/thiserror/latest/thiserror/index.html): 自定义, 丰富的错误处理, 适用于库级别。
- [snafu](https://docs.rs/snafu/latest/snafu/index.html): 更细粒度的错误管理。

需要注意 `Result<T, E>` 的大小。

# 日志处理

- [tracing](https://docs.rs/tracing/latest/tracing/index.html)
- [tracing-subscriber](https://docs.rs/tracing_subscriber/latest/tracing_subscriber/index.html)

# 宏

- [derive_builder](https://docs.rs/derive_builder/latest/derive_builder/index.html)
- [derive_more](https://docs.rs/derive_more/latest/derive_more/index.html)
- [strum](https://docs.rs/strum/latest/strum/index.html)
- [darling](https://docs.rs/darling/latest/darling/index.html)

# 数据转换

- [serde](https://docs.rs/serde/latest/serde/index.html) 生态

# 异步运行时

- [tokio](https://docs.rs/tokio/latest/tokio/index.html) 生态
- [tokio-stream](https://docs.rs/tokio-stream/latest/tokio_stream/index.html) 处理 stream
- [tokio-util](https://docs.rs/tokio-util/latest/tokio_util/index.html) tokio 实用工具
- [bytes](https://docs.rs/bytes/latest/bytes/index.html) 高效处理字节流
- [prost](https://docs.rs/prost/latest/prost/index.html) protobuf 序列化
- [tokio-console](https://docs.rs/tokio-console/latest/tokio_console/index.html) 调试工具
- [axum](https://docs.rs/axum/latest/axum/index.html) Web Framework
- [loom](https://docs.rs/loom/latest/loom/index.html) 测试工具
- [tracing](https://docs.rs/tracing/latest/tracing/index.html) 日志处理

# 应用开发

[tower](https://docs.rs/tower/latest/tower/index.html) 生态

# 关系型数据库

[sqlx](https://docs.rs/sqlx/latest/sqlx/index.html) 生态

```bash
docker run -d -p16686:16686 -p4317:4317 -e COLLECTOR_OTLP_ENABLED=true jaegertracing/all-in-one:latest
```
