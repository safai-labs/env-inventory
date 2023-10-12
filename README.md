
# Env-Inventory: Centralized Environment Variable Management

`env-inventory` is a Rust library designed to simplify and consolidate the process of fetching, managing, and validating parameters from the environment. Whether you're building a standalone application or integrating multiple crates, `env-inventory` ensures that all required environment variables are set and accessible in a unified manner. It also seamlessly integrates with TOML files, allowing for hierarchical configurations and fallbacks.

## Key Benefits

- 🌐 **Centralized Registration**: Any crate or module can register its required environment variables, ensuring that all dependencies on environment variables are explicitly declared and checked.
- 🌍 **Unified Access**: Streamline and standardize the way parameters are fetched from the environment, ensuring consistent access across your application.
- 📁 **TOML Support**: Read and merge configurations directly from TOML files. This is especially useful for managing different configurations for development, testing, and production environments.
- ✅ **Validation**: Before your application starts its main logic, validate and ensure that all required environment variables are set, reducing the risk of runtime failures due to misconfigurations.
- 🔍 **Transparency**: Functions like `list_all_vars` and `dump_all_vars` allow for introspection, aiding in debugging and ensuring all necessary environment variables have been registered.

## Getting Started

### Installation

Add `env-inventory` to your `Cargo.toml`:

```toml
[dependencies]
env-inventory = "0.2" # Check crates.io for the latest version
```

Usage

   Register the environment variables your application or crate depends on:

