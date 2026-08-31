## Why

The repository currently relies on manual maintenance to keep Rust crates and GitHub Actions current. Dependabot provides free, GitHub-native updates for both ecosystems with less operational and configuration overhead than broader dependency-management services.

## What Changes

- Configure scheduled Dependabot version updates for Cargo dependencies and the committed lockfile.
- Configure scheduled Dependabot version updates for GitHub Actions used by workflows and local composite actions.
- Group or limit routine update pull requests to keep maintenance noise manageable while preserving separate handling where appropriate.
- Ensure dependency-only pull requests trigger the repository's normal test, lint, formatting, coverage, and documentation checks.
- Continue maintaining the pinned `cargo-dist` version manually because Dependabot does not manage arbitrary values in `dist-workspace.toml`.

## Capabilities

### New Capabilities

None. This change adds repository maintenance automation and does not alter application behavior.

### Modified Capabilities

None.

## Impact

- Adds Dependabot configuration under `.github/`.
- Updates CI path filters so changes to Cargo manifests and lockfiles are validated.
- Creates automated pull requests that modify `Cargo.toml`, `Cargo.lock`, and GitHub Actions references.
- Introduces no runtime API, CLI, dependency, or user-facing behavior changes.
- Leaves `dist-workspace.toml` and its generated release workflow outside Dependabot's Cargo and GitHub Actions version coverage where versions are represented as arbitrary values.
