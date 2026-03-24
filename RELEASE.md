# Releasing maml to crates.io

## Prerequisites

1. A [crates.io](https://crates.io) account linked to your GitHub
2. An API token from [crates.io/settings/tokens](https://crates.io/settings/tokens)
3. Login locally:
   ```sh
   cargo login <your-token>
   ```

## Pre-release checklist

1. **Update version** in `Cargo.toml`:
   ```toml
   version = "0.2.0"
   ```

2. **Run tests and lints**:
   ```sh
   cargo test
   cargo clippy
   ```

3. **Dry-run publish** to catch issues early:
   ```sh
   cargo publish --dry-run
   ```

4. **Commit and tag**:
   ```sh
   git add -A
   git commit -m "Release v0.2.0"
   git tag v0.2.0
   git push origin main --tags
   ```

## Publish

```sh
cargo publish
```

The crate will be available at [crates.io/crates/maml](https://crates.io/crates/maml) and docs will auto-build at [docs.rs/maml](https://docs.rs/maml).

## Versioning

Follow [SemVer](https://semver.org):

- **Patch** (0.1.x): bug fixes, no API changes
- **Minor** (0.x.0): new features, backwards-compatible
- **Major** (x.0.0): breaking API changes
