# ONES Toolkit

[![Release](https://forgejo.cloud1ful.com/tools/ones/actions/workflows/release.yml/badge.svg)](https://forgejo.cloud1ful.com/tools/ones/actions/workflows/release.yml)
[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)

ONES 的非官方第三方集成工具，提供：

- HTTP JSON API 和 OpenAPI 文档
- MCP Streamable HTTP 和 stdio
- 命令行访问工作台、Issue、附件、代码关联和状态流转
- 发布适用于 Linux、macOS、Windows 的预编译二进制文件

项目独立开发，与 ONES 官方无隶属、赞助或授权关系。

## 部署

### 使用发布版二进制

从 [Releases](https://forgejo.cloud1ful.com/tools/ones/releases) 下载对应平台的二进制文件和 `.sha256` 校验文件，然后配置：

```bash
export ONES_BASE_URL="https://ones.example.test"
export ONES_TEAM_UUID="team-demo-001"
```

启动 HTTP API 和 MCP 服务：

```bash
chmod +x ones-linux-amd64
./ones-linux-amd64 serve --addr 0.0.0.0:3000
```

默认地址为 `http://127.0.0.1:3000`，MCP 地址为 `/mcp`，OpenAPI 文档地址为 `/openapi.json`。可通过 `LISTEN_ADDR` 和 `MCP_PATH` 修改默认值。

HTTP 请求使用以下请求头进行 ONES 认证：

```text
x-ones-email: user@example.test
x-ones-password: your-password
```

### 从源码运行

需要 Rust stable：

```bash
git clone https://forgejo.cloud1ful.com/tools/ones.git
cd ones
cargo run --release -- serve --addr 0.0.0.0:3000
```

### 配置 stdio MCP

stdio MCP 还需要 ONES 账号密码：

```bash
export ONES_EMAIL="user@example.test"
export ONES_PASSWORD="your-password"
cargo run --release -- stdio
```

## 发布

推送匹配 `v*` 的标签会触发 Forgejo Actions，自动运行测试并发布多平台二进制文件：

```bash
git tag vX.Y.Z
git push origin vX.Y.Z
```

也可以在 Actions 页面手动执行 `workflow_dispatch`，但需要填写一个已存在的标签。

MIT License
