# tia — Terraform Import Accelerator

Discover existing cloud resources and generate Terraform import blocks to bring them under IaC management.

## Installation

```bash
cargo install tia
```

## Usage

### Discover resources

List all importable resources in a Cloudflare zone:

```bash
export CLOUDFLARE_API_TOKEN="your-api-token"
export CLOUDFLARE_ZONE_ID="your-zone-id"

tia cloudflare discover
```

### Generate import blocks

Generate Terraform `import {}` blocks for discovered resources:

```bash
tia cloudflare generate
```

### Diff against Terraform state

Compare discovered cloud resources against an existing Terraform state file to find unmanaged resources:

```bash
tia cloudflare diff
```

## Environment Variables

| Variable               | Description                                              |
| ---------------------- | -------------------------------------------------------- |
| `CLOUDFLARE_API_TOKEN` | **Required.** Cloudflare API token for authentication    |
| `CLOUDFLARE_ZONE_ID`   | Optional zone ID to scope discovery to a single zone     |
| `RUST_LOG`             | Control log verbosity (`debug`, `info`, `warn`, `error`) |

## Development

```bash
# Build
cargo build

# Run tests
cargo test

# Lint
cargo clippy

# Format check
cargo fmt --check
```
