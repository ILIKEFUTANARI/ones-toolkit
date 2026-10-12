# ONES Toolkit

[English](README.md) | 简体中文

[![Release](https://forgejo.cloud1ful.com/tools/ones/actions/workflows/release.yml/badge.svg)](https://forgejo.cloud1ful.com/tools/ones/actions/workflows/release.yml)
[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)

ONES 的非官方第三方集成工具，提供：

- HTTP JSON API 和 OpenAPI 文档
- MCP Streamable HTTP 和 stdio
- 命令行访问工作台、Issue、附件、代码关联、状态流转和统一工时
- 旧版 `/project/api/project/team/:teamUUID` REST 端点（字段、状态流转、任务 info）和旧版 `items/graphql` 端点的兼容入口
- 发布适用于 Linux、macOS、Windows 的预编译二进制文件

项目独立开发，与 ONES 官方无隶属、赞助或授权关系。

## Cargo 特性

适配器按特性隔离，库核心不依赖任何服务器、CLI 或 schema 依赖：

| 特性 | 作用 | 引入的直接依赖 |
| --- | --- | --- |
| `http` | axum HTTP JSON API 与 OpenAPI 文档 | `axum`、`utoipa`、`utoipa-axum` |
| `mcp` | MCP 工具与资源（含 stdio 传输） | `rmcp`、`schemars` |
| `mcp-http` | MCP Streamable HTTP 传输（包含 `mcp`） | `rmcp` 的 HTTP 服务端特性 |
| `cli` | `ones` 二进制（包含 `http` 与 `mcp-http`） | `clap`、`tracing-subscriber` |
| `default` | `http` + `mcp` + `cli` | — |

只用业务逻辑：

```toml
ones-toolkit = { version = "0.2", default-features = false }
```

`default-features = false` 时不启用 `axum`、`rmcp`、`utoipa`、`utoipa-axum`、`clap`、`tracing-subscriber`、`schemars`，`ones` 二进制也不会构建；`OnesService`、`AppConfig`、`AppError` 以及全部共享 DTO 保持可用，`reqwest` 带来的 `http`、`hyper`、`tokio` 属于正常传递依赖。

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

## 工时

工时接口统一使用小时小数，Issue 可用 UUID 或显示 ID 标识。Issue 明细查询的人员、起止日期可省略；跨 Issue 报表必须提供 `from` 和 `to`。

HTTP 入口：

```text
GET    /api/work-hours/issue
GET    /api/work-hours/report
POST   /api/work-hours
PATCH  /api/work-hours/remaining
DELETE /api/work-hours
```

MCP 工具：`list_issue_work_hours`、`query_work_hour_report`、`add_work_hour`、`update_remaining_hours`、`delete_work_hour`。

CLI 示例：

```bash
ones work-hour list --issue-uuid issue-demo-001
ones work-hour report --from 2026-07-01 --to 2026-07-31
ones work-hour add --input work-hour.json
ones work-hour update-remaining --issue-uuid issue-demo-001 --remaining-hours 0
ones work-hour delete --display-id DEMO-1234 manhour-demo-recorded
```

新增和修改默认使用当前登录用户。删除必须传入目标 Issue 和精确的工时记录 `key`；返回结果中的人员姓名默认脱敏。

## 旧版接口

在用户环境无法使用新版 Open API v2 时，下列旧版公开接口可用：

- `GET /api/issues/{issue_uuid}/task-info`：读取旧版 `task/{uuidOrNumber}/info`，返回任务的基本字段（不含自定义字段）。
- `GET /api/issues/{issue_uuid}/transitions`：读取旧版 `task/{uuid}/transitions`，列出可用的状态流转。
- 字段定义、旧版 `items/graphql` GraphQL 等能力由现有统一接口复用。

MCP 工具对应：`get_issue_task_info`、`list_issue_transitions`。CLI 对应：

```bash
ones issue task-info issue-demo-001
ones issue transitions issue-demo-001
```

字段读取与状态流转在公开端点不可用时会自动回退到内部兼容端点；其他状态码（401、403、400、429、5xx、网络错误）按原状态上抛，不触发回退。

### 从源码运行

需要 Rust 1.88 或更高版本（`rust-version` 声明的下限，由依赖集合决定）：

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

### 状态流转输入

`issue transition-preview` 和 `issue transition` 通过 `--input PATH|-` 读取 JSON。`field_values[].value` 会保留 JSON 结构，支持字符串、单个对象和对象数组，例如：

```json
{
  "transition_uuid": "transition-demo-001",
  "field_values": [
    {
      "field_uuid": "field057",
      "value": [
        {"uuid": "comment-demo-001", "content": "FAT 回归评论"}
      ]
    }
  ]
}
```

## 发布

推送匹配 `v*` 的标签会触发 Forgejo Actions，自动运行测试并发布多平台二进制文件：

```bash
git tag vX.Y.Z
git push origin vX.Y.Z
```

也可以在 Actions 页面手动执行 `workflow_dispatch`，但需要填写一个已存在的标签。

MIT License
