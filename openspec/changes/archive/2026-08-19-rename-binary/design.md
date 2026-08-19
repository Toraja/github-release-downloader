## Context

See proposal.md - Why. The package currently has no `[[bin]]` section, so Cargo derives the binary name from the package name `github-release-downloader`. Distribution uses cargo-dist, whose installer artifact name (`github-release-downloader-installer.sh`) derives from the package name, and the install URL is published in `README.md` and depended on by existing users.

## Goals / Non-Goals

**Goals:**
- Produce a binary named `ghrls` without disrupting the existing install URL.

**Non-Goals:**
- Renaming the Cargo package or the GitHub repository.
- Changing any CLI arguments or runtime behavior.

## Decisions

### Rename via `[[bin]]`, keep the package name

**Decision:** Add an explicit `[[bin]] name = "ghrls"` section pointing at `src/main.rs`, leaving `[package] name = "github-release-downloader"` unchanged.

**Rationale:** In Cargo the binary name and package name are independent. Overriding only the binary name gives the desired `ghrls download` UX while the package name continues to drive the cargo-dist installer artifact name and URL — so `github-release-downloader-installer.sh` and the README install command keep working unchanged.

**Alternative considered:** Rename the package (`[package] name = "ghrls"`). Rejected because it changes the cargo-dist installer artifact name to `ghrls-installer.sh`, breaking the published install URL and existing users' scripts for no functional benefit.

## Risks / Trade-offs

- **Users who invoke `github-release-downloader` directly break** → This is the intended rename; call it out in README and the release notes. No automatic alias is provided.
- **Stale references to the old binary name in docs/ADRs** → Update `README.md` and the `User-Agent` string; leave archived openspec/ADR history untouched since it records past state.
