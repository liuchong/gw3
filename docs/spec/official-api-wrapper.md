# Official API Wrapper Specification

`gw3` wraps the Guild Wars 2 official API v2 and Guild Wars 2 Wiki read APIs.
Version `0.0.1` exposes a Rust library plus one binary: `gw3` for command-line usage and MCP clients.

## Product Boundary

The first release is read-only. It can query public game data, authenticated account data, and public Wiki pages. It must not read the game process, automate gameplay, persist secrets, or expose unfinished Windows internal features.

The next stage expands public API coverage by introducing a built-in public endpoint registry. The registry includes official `/v2.json` routes where `active=true` and `auth=false`, and it excludes authenticated routes from the public command surface.

## Code Architecture

The crate is intentionally single-package but not flat. Responsibilities are split by layer:

- `src/config.rs` and `src/error.rs`: shared configuration and stable error types.
- `src/gw2/`: Guild Wars 2 official API domain client, including request models, endpoint metadata, public endpoint registry, path normalization, and HTTP behavior.
- `src/wiki/`: Guild Wars 2 Wiki MediaWiki client.
- `src/api/`: compatibility facade that re-exports the public API types from `gw2`, `config`, and `error`.
- `src/cli/`: command-line argument definitions, command runner, and JSON output adapter.
- `src/mcp/`: MCP server construction, tool parameter models, and tool implementations.
- `src/bin/`: thin binary entrypoints only.

CLI and MCP must not duplicate GW2 or Wiki request behavior. They delegate to `gw2` and `wiki`.

## API Client Behavior

The core API client sends requests to `https://api.guildwars2.com` by default. Tests may override the base URL with a local mock server.

Configuration fields:

- `base_url`: default official API origin.
- `wiki_base_url`: default Guild Wars 2 Wiki API origin.
- `lang`: optional language value, one of `en`, `es`, `de`, `fr`, or `zh`.
- `schema_version`: optional API schema version, usually `latest`.
- `api_key`: optional key read from command argument or `GW2_API_KEY`.
- `timeout`: request timeout in seconds.

Decision rules:

- Authenticated endpoints use `Authorization: Bearer <key>`.
- Query-string `access_token` is not used by default because it can leak through URLs and logs.
- A command or MCP tool that requires a key must fail locally when no key is available.
- `ids` requests are chunked into groups of at most 200 IDs.
- `lang` and `v` are sent as query parameters when present.
- API errors are normalized into stable error kinds: invalid input, missing key, forbidden, not found, partial content, rate limited, disabled endpoint, upstream failure, transport failure, and JSON failure.
- Error messages must not include the API key.

## Public Endpoint Registry

The crate maintains a static public endpoint registry derived from the official API route list.

Decision rules:

- Registry entries cover routes with `active=true` and `auth=false`.
- Routes with `auth=true` stay accessible through low-level generic API calls, but are rejected by the `public` command family until authenticated workflow coverage is explicitly added.
- Routes with `active=false` are recorded as unavailable and are not exposed as default public commands.
- Registry entries record a stable user-facing `key`, canonical official `path`, whether `lang` is supported, and how the route should be accessed:
  - collection routes that support base list and optional `id` or `ids`
  - singleton routes that should be fetched directly at the base path
  - call-only routes that need custom query or path parameters
- CLI and MCP must use the same registry and validation rules.

## CLI Behavior

The `gw3` binary exposes these commands:

- `gw3 api routes`: fetch `/v2.json` and show known route metadata.
- `gw3 api get <path>`: call an official API path with optional `--lang`, `--schema-version`, `--id`, `--ids`, `--page`, and `--page-size`.
- `gw3 token info`: call `/v2/tokeninfo` using the configured API key.
- `gw3 item get --ids <ids>`: fetch `/v2/items`.
- `gw3 item prices --ids <ids>`: fetch `/v2/commerce/prices`.
- `gw3 account info`: fetch `/v2/account`.
- `gw3 character list`: fetch `/v2/characters`.
- `gw3 public routes`: return the static registry of active public endpoints.
- `gw3 public list <key>`: call the base path for a public collection route.
- `gw3 public get <key>`: call a public route with optional `--id`, `--ids`, `--lang`, and `--schema-version`.
- `gw3 public all <key>`: call a public route with `ids=all`.
- `gw3 public page <key>`: call a public route with `--page` and `--page-size`.
- `gw3 public call <key>`: call a public route with explicit `--param name=value` and `--query name=value`.
- `gw3 wiki search <query>`: search Guild Wars 2 Wiki pages.
- `gw3 wiki page <title>`: fetch a plain-text page summary and URL.
- `gw3 mcp serve`: serve MCP over standard input/output.

CLI output is JSON by default so it can be piped into other tools.

## MCP Behavior

The `gw3 mcp serve` command serves MCP over standard input/output. It exposes:

- `gw2_api_request`: generic official API request.
- `gw2_token_info`: token permission inspection.
- `gw2_item_lookup`: item lookup.
- `gw2_item_prices`: trading post price lookup.
- `gw2_account_summary`: account summary.
- `gw2_character_list`: character list.
- `gw2_public_routes`: return the public endpoint registry.
- `gw2_public_list`: call the base path for a public collection route.
- `gw2_public_get`: call a public route with optional `id` or `ids`.
- `gw2_public_all`: call a public route with `ids=all`.
- `gw2_public_page`: call a public route with paging parameters.
- `gw2_public_call`: call a public route with explicit path and query parameters.
- `gw2_wiki_search`: Wiki search.
- `gw2_wiki_page`: Wiki page summary.

Each MCP tool delegates to the same core functions used by CLI commands.

## Wiki Behavior

Wiki calls use `https://wiki.guildwars2.com/api.php` by default.

Supported read operations:

- Search pages via MediaWiki `query&list=search`.
- Read page summary, full URL, and metadata via `query&prop=extracts|info`.
- Return normalized JSON suitable for CLI and MCP clients.

The first release does not parse complex wikitext templates into stable typed data.

## BDD Acceptance Scenarios

- Given no API key, when `gw3 item get --ids 19684` runs against a public mock endpoint, then it returns item JSON.
- Given no API key, when `gw3 account info` runs, then it fails locally with a missing key error and sends no HTTP request.
- Given an API key, when `gw3 token info` runs, then it sends `Authorization: Bearer <key>` and returns token JSON without printing the key.
- Given no API key, when `gw3 public routes` runs, then it returns active public endpoints and excludes authenticated routes such as `/v2/account`.
- Given no API key, when `gw3 public get skins --ids 4674 --lang zh` runs, then it returns skin JSON and sends no Authorization header.
- Given no API key, when `gw3 public call recipes_search --query input=46747` runs, then it sends the custom query to `/v2/recipes/search`.
- Given no API key, when `gw3 public call pvp_season_leaderboards --param id=2v2-season --param board=ladder --param region=eu` runs, then it expands the path template locally before the HTTP request.
- Given an authenticated route key is requested through `gw3 public get`, when the registry marks it as authenticated-only, then the command fails locally before any HTTP request.
- Given an inactive route key is requested through `gw3 public get`, when the registry marks it as inactive, then the command fails locally before any HTTP request.
- Given more than 200 IDs, when an item lookup runs, then the client splits the request and merges successful JSON arrays.
- Given an upstream `429`, when any API command runs, then it returns a rate-limit error without retrying forever.
- Given an MCP client calls `gw2_item_prices`, when the arguments match `gw3 item prices`, then both entry points return the same JSON semantics.
- Given an MCP client calls `gw2_public_get`, when the arguments match `gw3 public get`, then both entry points return the same JSON semantics.
- Given a user installs the crate, when they inspect installed binaries, then the project exposes only the `gw3` binary and MCP is reached through `gw3 mcp serve`.
- Given a Wiki query, when `gw3 wiki search Legendary armor` runs, then it returns matching page titles and URLs.

## Verification

Release `0.0.1` is complete only after:

- `cargo fmt --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test --all-targets`
- `cargo publish --dry-run`
