# Env-Inventory: Environment Variable Management

`env-inventory` is a Rust library designed to manage and consolidate the process of fetching parameters from the environment. It offers a flexible system to work with environment variables and configurations stored in TOML files.

## Features

- 🌍 **Unified Access**: Streamline the way parameters are fetched from the environment.
- 📁 **TOML Support**: Read and merge configurations directly from TOML files, allowing hierarchical configurations.
- ✅ **Validation**: Validate and ensure that required environment variables are set.

## Getting Started

### Installation

Add `env-inventory` to your `Cargo.toml`:

```toml
[dependencies]
env-inventory = "0.1.0" # Check crates.io for the latest version
```

### Usage

1. Define required environment variables using the `RequiredVar` struct.

```rust
inventory::submit!(RequiredVar::new("DATABASE_URL", None));
```

2. Load and validate environment variables from your TOML configurations:

```rust
let paths = ["path/to/settings.toml"];
env_inventory::load_and_validate_env_vars(&paths, "env").unwrap();
```

## Error Handling

The library provides an `EnvInventoryError` enum to handle various error types such as:
- Reading or parsing the settings file.
- Missing required environment variables.

## Testing

We provide a suite of tests to validate core functionalities. Run tests with:

```bash
cargo test
```

## Contributing

Pull requests are welcome. Please ensure that your PR passes all the tests before submitting.

## License

[MIT](LICENSE)

---

Feel free to modify the README as needed to tailor it to your project's specific needs and details!

