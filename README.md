# ONES Toolkit

English | [简体中文](README.zh-CN.md)

[![Release](https://forgejo.cloud1ful.com/tools/ones/actions/workflows/release.yml/badge.svg)](https://forgejo.cloud1ful.com/tools/ones/actions/workflows/release.yml)
[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)

Unofficial third-party ONES integration providing:

- An HTTP JSON API with a generated OpenAPI document
- MCP over Streamable HTTP and stdio
- A CLI for workbench views, issues, attachments, code links, status transitions, and work hours
- Compatibility entry points for the legacy `/project/api/project/team/:teamUUID` REST endpoints (fields, status transitions, task info) and the legacy `items/graphql` endpoint
- Prebuilt binaries for Linux, macOS, and Windows

Independently developed; not affiliated with, sponsored, authorized, or endorsed by ONES or its owners.

## Cargo features

Adapters are feature-gated, so the library core never pulls in a server, CLI, or schema dependency:

| Feature | Enables | Direct dependencies added |
| --- | --- | --- |
| `http` | axum HTTP JSON API and OpenAPI document | `axum`, `utoipa`, `utoipa-axum` |
| `mcp` | MCP tools and resources, including the stdio transport | `rmcp`, `schemars` |
| `mcp-http` | MCP Streamable HTTP transport (implies `mcp`) | rmcp's HTTP server features |
| `cli` | The `ones` binary (implies `http` and `mcp-http`) | `clap`, `tracing-subscriber` |
| `default` | `http` + `mcp` + `cli` | — |

Consume only the business logic:

```toml
ones-toolkit = { version = "0.2", default-features = false }
```

With `default-features = false`, `axum`, `rmcp`, `utoipa`, `utoipa-axum`, `clap`, `tracing-subscriber`, and `schemars` stay disabled and the `ones` binary is not built, while `OnesService`, `AppConfig`, `AppError`, and every shared DTO remain available. The `http`, `hyper`, and `tokio` crates that `reqwest` brings in remain legitimate transitive dependencies.

Select only what you need:

| Dependency | Behaviour |
| --- | --- |
| `default-features = false` | Core business layer only |
| `features = ["http"]` | HTTP API without any MCP transport |
| `features = ["mcp"]` | MCP tools over stdio, without the HTTP server |
| `features = ["mcp-http"]` | MCP Streamable HTTP transport |
| default features | Full HTTP, MCP, and CLI surface |

## Deploy

### Released binary

Download the binary and its `.sha256` checksum for your platform from [Releases](https://forgejo.cloud1ful.com/tools/ones/releases), then configure:

```bash
export ONES_BASE_URL="https://ones.example.test"
export ONES_TEAM_UUID="team-demo-001"
```

Start the HTTP API and MCP server:

```bash
chmod +x ones-linux-amd64
./ones-linux-amd64 serve --addr 0.0.0.0:3000
```

The default address is `http://127.0.0.1:3000`, MCP is served at `/mcp`, and the OpenAPI document at `/openapi.json`. Override the defaults with `LISTEN_ADDR` and `MCP_PATH`.

HTTP requests authenticate with these headers:

```text
x-ones-email: user@example.test
x-ones-password: your-password
```

### From source

Rust 1.88 or newer is required (the floor declared by `rust-version` and set by
the dependency set):

```bash
git clone https://forgejo.cloud1ful.com/tools/ones.git
cd ones
cargo run --release -- serve --addr 0.0.0.0:3000
```

### stdio MCP

The stdio MCP server also needs ONES account credentials:

```bash
export ONES_EMAIL="user@example.test"
export ONES_PASSWORD="your-password"
cargo run --release -- stdio
```

### Status transition input

`issue transition-preview` and `issue transition` read JSON from `--input PATH|-`. `field_values[].value` keeps its JSON shape and accepts a string, a single object, or an array of objects:

```json
{
  "transition_uuid": "transition-demo-001",
  "field_values": [
    {
      "field_uuid": "field057",
      "value": [
        {"uuid": "comment-demo-001", "content": "regression note"}
      ]
    }
  ]
}
```

## Work hours

Work-hour endpoints use fractional hours and accept an issue UUID or display ID. Owner and date bounds on the issue-detail query are optional; the cross-issue report requires `from` and `to`.

HTTP entry points:

```text
GET    /api/work-hours/issue
GET    /api/work-hours/report
POST   /api/work-hours
PATCH  /api/work-hours/remaining
DELETE /api/work-hours
```

MCP tools: `list_issue_work_hours`, `query_work_hour_report`, `add_work_hour`, `update_remaining_hours`, `delete_work_hour`.

CLI examples:

```bash
ones work-hour list --issue-uuid issue-demo-001
ones work-hour report --from 2026-07-01 --to 2026-07-31
ones work-hour add --input work-hour.json
ones work-hour update-remaining --issue-uuid issue-demo-001 --remaining-hours 0
ones work-hour delete --display-id DEMO-1234 manhour-demo-recorded
```

Creates and updates default to the signed-in user. A delete must name the target issue and the exact work-hour entry `key`; person names in responses are redacted by default.

## Legacy endpoints

When the newer Open API v2 is unavailable in your environment, these legacy public endpoints work:

- `GET /api/issues/{issue_uuid}/task-info` reads legacy `task/{uuidOrNumber}/info` and returns the basic task fields (custom fields excluded).
- `GET /api/issues/{issue_uuid}/transitions` reads legacy `task/{uuid}/transitions` and lists the available status transitions.
- Field definitions and the legacy `items/graphql` GraphQL surface are reused from the unified endpoints.

Matching MCP tools are `get_issue_task_info` and `list_issue_transitions`. Matching CLI commands:

```bash
ones issue task-info issue-demo-001
ones issue transitions issue-demo-001
```

Field reads and status transitions automatically fall back to the internal compatibility endpoints when the public ones are unavailable. Other statuses (401, 403, 400, 429, 5xx, network errors) propagate unchanged and never trigger a fallback.

## Release

Pushing a `v*` tag triggers Forgejo Actions, which runs the feature matrix and tests, then publishes prebuilt binaries:

```bash
git tag vX.Y.Z
git push origin vX.Y.Z
```

You can also run `workflow_dispatch` manually from the Actions page, but you must supply an existing tag.

MIT License