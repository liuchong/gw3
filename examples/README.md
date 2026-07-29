# Examples

These examples are runnable Cargo examples for the `gw3` library API. They call live Guild Wars 2 official API and Wiki endpoints, so the printed JSON is real upstream output.

Run the public endpoint registry example:

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

The `public_api` example demonstrates the new public registry layer by listing built-in routes, fetching public resources through registry keys, and calling a query-driven public endpoint.

These examples intentionally avoid account, character, inventory, wallet, guild, PvP, WvW, and trading post history endpoints because those require `GW2_API_KEY`. Add authenticated examples only after a real test key is available.
