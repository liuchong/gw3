# Quick Demo

These examples use the single `gw3` binary. Public examples do not require a Guild Wars 2 API key.

## Install Locally

```sh
cargo install --path .
```

For development without installing:

```sh
cargo run --quiet -- api get /v2/build
```

Expected shape:

```json
{
  "id": 204489
}
```

## Public Item Lookup

Fetch one item in Chinese:

```sh
gw3 item get --ids 19684 --lang zh
```

Expected shape:

```json
[
  {
    "id": 19684,
    "name": "秘银锭",
    "type": "CraftingMaterial"
  }
]
```

## Trading Post Prices

Fetch current buy and sell orders:

```sh
gw3 item prices --ids 19684
```

Expected shape:

```json
[
  {
    "id": 19684,
    "buys": {
      "quantity": 1384825,
      "unit_price": 75
    },
    "sells": {
      "quantity": 2449558,
      "unit_price": 79
    }
  }
]
```

Live quantities and prices change over time.

## Wiki Search

Search the official Guild Wars 2 Wiki:

```sh
gw3 wiki search "Legendary armor"
```

Expected shape:

```json
{
  "query": {
    "search": [
      {
        "title": "Legendary armor"
      }
    ]
  }
}
```

## Authenticated Account Data

Private account endpoints require an API key from `https://account.arena.net/applications`.

```sh
export GW2_API_KEY="your-api-key"
gw3 token info
gw3 account info
gw3 character list
```

`gw3` sends the key with `Authorization: Bearer <key>` and does not store it.

## MCP Service Mode

MCP is served through the same `gw3` binary:

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
