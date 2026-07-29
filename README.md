# gw3

`gw3` is a Rust wrapper for the Guild Wars 2 official API v2 and Guild Wars 2 Wiki read APIs.

Version `0.0.1` ships one publishable crate with:

- `gw3`: the single binary entrypoint for command-line usage and MCP service mode.
- `gw3`: a Rust library used by the binary entrypoint.

The first release is read-only. It does not read the game process, automate gameplay, or implement Windows internal interfaces.

## Install

From the repository:

```sh
cargo install --path .
```

After publishing:

```sh
cargo install gw3
```

## API Key

Public game data does not need an API key. Account, character, inventory, wallet, trading post history, PvP, WvW, and guild private data require a Guild Wars 2 API key.

Create a key at `https://account.arena.net/applications`.

Use an environment variable:

```sh
export GW2_API_KEY="your-api-key"
```

Or pass it to supported commands:

```sh
gw3 --api-key "your-api-key" token info
```

`gw3` sends keys via `Authorization: Bearer <key>` and does not persist them.

## CLI Examples

List official API routes:

```sh
gw3 api routes
```

Call a public endpoint directly:

```sh
gw3 api get /v2/items --ids 19684 --lang zh
```

List built-in active public endpoints:

```sh
gw3 public routes
```

Use the public registry to fetch resources:

```sh
gw3 public get skins --ids 4674 --lang zh
gw3 public get achievements --ids 1
gw3 public page maps --page 0 --page-size 5 --lang zh
gw3 public call recipes_search --query input=46747
```

High-frequency shortcuts remain available:

```sh
gw3 item get --ids 19684,19721 --lang zh
gw3 item prices --ids 19684
```

Inspect the configured API key:

```sh
gw3 token info
```

Read account data:

```sh
gw3 account info
gw3 character list
```

Search Guild Wars 2 Wiki:

```sh
gw3 wiki search "Legendary armor"
gw3 wiki page "Legendary armor"
```

All CLI commands print JSON by default. Runnable Rust examples are available in [examples/](examples/).

## MCP Usage

Run the MCP server:

```sh
gw3 mcp serve
```

Example MCP client configuration:

```json
{
  "mcpServers": {
    "gw3": {
      "command": "gw3",
      "args": ["mcp", "serve"],
      "env": {
        "GW2_API_KEY": "your-api-key"
      }
    }
  }
}
```

The server exposes these tools:

- `gw2_api_request`
- `gw2_token_info`
- `gw2_item_lookup`
- `gw2_item_prices`
- `gw2_account_summary`
- `gw2_character_list`
- `gw2_public_routes`
- `gw2_public_list`
- `gw2_public_get`
- `gw2_public_all`
- `gw2_public_page`
- `gw2_public_call`
- `gw2_wiki_search`
- `gw2_wiki_page`

## Development

Run the full verification suite:

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo publish --dry-run
```

Integration tests live in `integration_test/` and use local mock servers instead of the live API.

## License

`gw3` is licensed under the [Zero Public License 1.0](https://license.pub/0pl/) through the repository `LICENSE` file.
