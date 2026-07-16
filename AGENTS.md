# Project Operating Rules

## Sensitive Example Data

- Treat every file under `example/` as potentially sensitive capture material. HAR files and ONES exports may contain real internal hostnames/IPs, issue IDs, repository paths, names, email addresses, auth headers, cookies, tokens, and signed download URLs.
- Never copy values from `example/` into Rust tests, fixtures, README files, OpenAPI examples, logs, or source code. Use synthetic values such as `issue-demo-001`, `DEMO-1234`, `example.test`, `git.example.test`, and `192.0.2.0/24` instead.
- Test fixtures must use fake identities and credentials only. Keep names like `Alice Example`, emails under reserved example domains, and tokens such as `mock-token`; do not use real employee, customer, project, repository, or issue data.
- Never commit raw HAR files, ONES JSON exports, signed attachment URLs, cookies, authorization headers, or any other credential-bearing capture. Keep capture files ignored and share only narrowly filtered, sanitized entries when needed for development.
- Before staging changes, review `git diff` and untracked files for email addresses, internal IPs/hostnames, real issue or repository identifiers, tokens, passwords, cookies, and `X-Amz-*` query parameters. Replace or remove them before commit.

## Verification

- When adding a behavior based on a real HAR, first reduce it to the smallest request/response contract and write synthetic fixtures for that contract.
- Do not weaken this rule because a file is ignored locally; ignored capture files remain sensitive and must not leak into generated artifacts or documentation.

## Three-Entry Contract Parity

- Keep ONES business logic and upstream request orchestration in `OnesService`; HTTP, MCP, and CLI layers only adapt transport input, authentication, output, and errors.
- A new user-facing capability must be reviewed across the service, HTTP route/OpenAPI contract, MCP tool/schema, CLI command, README, and tests in the same change.
- Reuse shared request and response DTOs across HTTP, MCP, and CLI. Add both `serde` serialization and the schema derives required by each public contract instead of duplicating payload structs.
- Treat the Rust OpenAPI annotations and shared DTOs as the HTTP contract source. `/openapi.json` and `ones openapi` are generated views; do not hand-edit a second OpenAPI document.
- Keep parity checks in tests: registered HTTP routes must remain documented, MCP tool names and schemas must remain stable, and CLI aliases must dispatch to the same service methods as canonical commands.
