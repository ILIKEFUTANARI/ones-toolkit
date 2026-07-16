# ONES Toolkit (Unofficial)

`ones-toolkit` is an unofficial third-party integration for ONES. It is
independently developed and is not affiliated with, sponsored, authorized, or
endorsed by ONES or its owners. ONES is used only to identify the compatible
product and remains the property of its respective trademark owner.

ONES Toolkit provides:

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

Required for business CLI commands:

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

CLI examples:

```bash
cargo run -- view list
cargo run -- get DEMO-1234
cargo run -- attachments issue-demo-001
cargo run -- issue transition-preview issue-demo-001 --input transition.json
cargo run -- --json issue get issue-demo-001
```

The CLI prints compact tables for list commands and readable JSON for detail
commands. Pass `--json` for machine-readable output. Transition input accepts a
file path or `-` for stdin. `ones openapi` prints the same generated document
served by `/openapi.json`.

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
- `GET /api/issues/{issue_uuid}/attachments?cursor=...`
- `GET /api/issues/{issue_uuid}/code-links?redact=...`
- `POST /api/issues/{issue_uuid}/branches`
- `GET /api/issues/{issue_uuid}/transitions`
- `POST /api/issues/{issue_uuid}/transition-preview`
- `POST /api/issues/{issue_uuid}/transition`
- `GET /api/issues/{issue_uuid}/activity?cursor=...&limit=...&redact=...`
- `GET /openapi.json`

## MCP Tools

- `list_workbench_views`
- `get_workbench_view`
- `list_workbench_issue_groups`
- `list_workbench_issues`
- `get_issue_by_display_id`
- `get_issue_detail`
- `list_issue_attachments`
- `list_issue_code_links`
- `create_issue_branch`
- `list_issue_transitions`
- `preview_issue_transition`
- `execute_issue_transition`
- `list_issue_activity`

## MCP Resources

- `ones://fields` returns the team field catalog.
- `ones://issues/{issue_uuid}/fields` returns fields applicable to one issue.

Field resources expose compact metadata such as field UUID, semantic type, value
type, reference type, and filter/sort/group capabilities. Core response fields
such as title, status, project, and assignee are resolved by the service from the
team catalog; callers and AI clients do not select those mappings. Team field
metadata is cached per credential set for five minutes.

## CLI Commands

Canonical command groups:

- `view list|get|groups|issues`
- `issue get|get-by-display-id|attachments|code-links|activity|transitions`
- `issue branch`
- `issue transition-preview|transition`

Shortcuts for common operations:

- `get <display_id>`
- `issues <view_uuid>`
- `attachments <issue_uuid>`
- `activity <issue_uuid>`

## Current scope

v1 is primarily read-oriented and prioritizes:

- direct issue lookup by display ID like `DEMO-1234`
- browsing issues through saved workbench views

Issue detail responses are compact and human-readable: they include core fields,
named non-empty custom fields, and related-resource summaries. ONES form layouts,
field-definition metadata, and raw activity payloads remain internal. Call
`list_issue_activity` only when the issue history is needed; it returns paged,
readable change entries.

Call `list_issue_attachments` when a design document or other file is needed. It
returns attachment metadata plus a temporary signed download URL (normally valid
for about one hour). Follow `page_info.next_cursor` while `has_next_page` is true,
and request the tool again to refresh an expired URL.

`list_issue_code_links` returns linked repositories (including `repo_uuid`),
branches, commits, and pull requests. Pass the returned repository UUID to
`create_issue_branch`.
`create_issue_branch` creates a branch through ONES for the selected linked
repository, using the supplied source and target branch names.

Status changes use a generic discovery/preview/execute flow. Call
`list_issue_transitions` first, then preview the selected transition with its
semantic worklog/test-scope values and any custom `field_values`; execute only
after the preview reports `valid: true`. The service revalidates the transition,
field permissions, required fields, and asynchronous result before writing.

The branch HTTP request body is:

```json
{
  "repo_uuid": "...",
  "source_branch": "prod",
  "target_branch": "feature/DEMO-1234"
}
```

The transition preview and execute HTTP request body is:

```json
{
  "transition_uuid": "transition-demo-001",
  "worklog": {
    "hours": 6.0,
    "description": "完成接口改造并补充异常处理",
    "start_at": "2026-07-14T09:00:00Z"
  },
  "test_scope": "覆盖主流程和异常分支",
  "field_values": [{
    "field_uuid": "field-demo-text",
    "value": "已完成接口回归"
  }]
}
```

Issue lists, details, and activity redact structured user identities by default.
Pass `redact=false` to the MCP tool or HTTP query to preserve names supplied by
ONES. This only disables this service's redaction; upstream `[[RDX:...]]` values
cannot be restored. Aliases are stable within one process, but may change after
a restart.

Keyword search and write operations beyond branch creation and the explicit
transition flow are intentionally out of scope for this version.
