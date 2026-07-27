# Examples

These examples are runnable Cargo examples for the `gw3` library API. They call live Guild Wars 2 official API and Wiki endpoints, so the printed JSON is real upstream output.

Run public Guild Wars 2 API calls:

```sh
cargo run --example public_api
```

Search the official Guild Wars 2 Wiki:

```sh
cargo run --example wiki_search -- "Legendary armor"
```

Read one official Wiki page:

```sh
cargo run --example wiki_page -- "Legendary armor"
```

These examples intentionally avoid account, character, inventory, wallet, guild, PvP, WvW, and trading post history endpoints because those require `GW2_API_KEY`. Add authenticated examples only after a real test key is available.
