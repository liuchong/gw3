# gw3 Agent Rules

本项目是 Rust 单 crate 项目，第一阶段只包装《激战 2》官方只读 API 和官方 Wiki 读取能力。

## Scope

- 只维护一个可发布到 crates.io 的 crate，包名为 `gw3`，版本从 `0.0.1` 开始。
- 同一个 crate 内只提供一个 `gw3` binary；普通 CLI 和 MCP 服务都必须挂在这个入口下。
- CLI 和 MCP 必须复用 `src/api`、`src/wiki` 中的同一套业务逻辑，入口层只做参数解析和协议适配。
- 本阶段不做 IDE 扩展、浏览器扩展、游戏插件 UI、Windows 内部接口、游戏进程读取或自动化游戏操作。

## Secrets

- API key 只能从 `--api-key` 或 `GW2_API_KEY` 读取。
- 不把 API key 写入仓库、本地配置、日志、错误输出或测试 fixture。
- `.local/` 和 `.profile/` 必须被 `.gitignore` 忽略。

## Testing

- 行为变更必须先更新长期规格和 BDD 场景，再写测试，再实现。
- 集成测试必须放在 `integration_test/`，并通过 `Cargo.toml` 的 `[[test]]` 接入。
- 网络相关测试默认使用本地 HTTP mock，不依赖真实官方服务或真实账号 key。
- 提交前必须运行 `cargo fmt --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test --all-targets` 和 `cargo publish --dry-run`。
