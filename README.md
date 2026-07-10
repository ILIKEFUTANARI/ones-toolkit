# ones

Read-only ONES issue wrapper with:

- HTTP JSON API for humans and scripts
- MCP over Streamable HTTP
- MCP over stdio

The project is released under the MIT license.

## Environment

Required for every mode:

- `ONES_BASE_URL`
  Example: `http://ones.example.com`
- `ONES_TEAM_UUID`

Required for `stdio` MCP:

- `ONES_EMAIL`
- `ONES_PASSWORD`

## Run

HTTP JSON API plus HTTP MCP:

```bash
cargo run -- serve
```

Default endpoint: `http://127.0.0.1:3000/mcp`.

Override the listener with `--addr` or `LISTEN_ADDR`:

```bash
cargo run -- serve --addr 127.0.0.1:8080
```

stdio MCP:

```bash
ONES_EMAIL=alex@example.test ONES_PASSWORD=dummy-password cargo run -- stdio
```

## GitHub Release

Pushing a tag that matches `v*` triggers the GitHub Actions workflow in
`.github/workflows/release.yml` and publishes release artifacts for:

- Linux amd64
- Linux arm64
- macOS arm64
- Windows amd64

Each release uploads one platform binary plus its `.sha256` file.

You can also run the workflow manually with `workflow_dispatch`, but it expects
an existing tag such as `v0.2.0`.

## HTTP API

Per-request auth headers:

- `x-ones-email`
- `x-ones-password`

Routes:

- `GET /api/workbench/views`
- `GET /api/workbench/views/{view_uuid}`
- `GET /api/workbench/views/{view_uuid}/groups`
- `GET /api/workbench/views/{view_uuid}/issues?group_value=...&cursor=...&limit=...&redact=...`
- `GET /api/issues/by-display-id/{display_id}?redact=...`
- `GET /api/issues/{issue_uuid}?redact=...`
- `GET /api/issues/{issue_uuid}/activity?cursor=...&limit=...&redact=...`
- `GET /openapi.json`

## MCP Tools

- `list_workbench_views`
- `get_workbench_view`
- `list_workbench_issue_groups`
- `list_workbench_issues`
- `get_issue_by_display_id`
- `get_issue_detail`
- `list_issue_activity`

## Current scope

v1 is read-only and prioritizes:

- direct issue lookup by display ID like `DEMO-1234`
- browsing issues through saved workbench views

Issue detail responses are compact and human-readable: they include core fields,
named non-empty custom fields, and related-resource summaries. ONES form layouts,
field-definition metadata, and raw activity payloads remain internal. Call
`list_issue_activity` only when the issue history is needed; it returns paged,
readable change entries.

Issue lists, details, and activity redact structured user identities by default.
Pass `redact=false` to the MCP tool or HTTP query to preserve names supplied by
ONES. This only disables this service's redaction; upstream `[[RDX:...]]` values
cannot be restored. Aliases are stable within one process, but may change after
a restart.

Keyword search and write operations are intentionally out of scope for this version.
