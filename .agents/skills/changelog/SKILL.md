---
name: changelog
description: Use when writing, updating, or reviewing a CHANGELOG, release notes, or version history following the Keep a Changelog format, including recording unreleased changes, cutting a release, or documenting breaking changes.
---

# Changelog

## Overview

A changelog is a curated, human-readable record of notable changes for each version, written for users and maintainers. Follow the [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) format plus the two project-specific rules below.

## When to Use

Use when creating a `CHANGELOG.md`, adding an entry for a change, cutting a release, or reviewing changelog edits.

Do not dump raw `git log` output. A changelog is curated: describe user-facing impact, not every commit.

## Core Rules (Keep a Changelog)

- Changelogs are for humans, not machines. Describe impact, not implementation.
- Keep an `## [Unreleased]` section at the top for changes not yet released.
- Every released version has its own section: `## [x.y.z] - YYYY-MM-DD` (ISO 8601 dates).
- Group entries under these headings, in this order, omitting empty ones:
  - `### Added` — new features
  - `### Changed` — changes to existing functionality
  - `### Deprecated` — soon-to-be-removed features
  - `### Removed` — now-removed features
  - `### Fixed` — bug fixes
  - `### Security` — vulnerability fixes
- Newest version first (reverse chronological).
- Use reference-style links for versions; define them in a footnote at the bottom.

## Project-Specific Rules

1. **Breaking changes** must be prefixed with `**Breaking:**` at the start of the entry line, regardless of which heading they fall under.
2. A `---` horizontal rule separates the oldest release section from the reference-style link footnote.

## Workflow

### Adding a change

1. Ensure an `## [Unreleased]` section exists at the top; create it if missing.
2. Pick the correct heading (Added / Changed / Deprecated / Removed / Fixed / Security).
3. Write one concise, user-facing bullet in the imperative or descriptive mood.
4. If the change breaks compatibility, prefix the bullet with `**Breaking:** `.

### Cutting a release

1. Rename `## [Unreleased]` to `## [x.y.z] - YYYY-MM-DD` using today's date.
2. Add a fresh empty `## [Unreleased]` section above it.
3. Update the reference-style links in the footnote:
   - `[Unreleased]` compares the new version to `HEAD`.
   - Add a `[x.y.z]` link for the new release.
4. Keep the `---` rule directly above the footnote, after the oldest release section.

## Example

```markdown
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- New `--timeout` flag to configure request timeouts.

## [2.0.0] - 2026-02-01

### Added

- Support for downloading pre-release assets.

### Changed

- **Breaking:** Rename `--repo` flag to `--repository`.

### Removed

- **Breaking:** Drop support for the deprecated `v1` config format.

### Fixed

- Handle rate-limit responses without crashing.

## [1.1.0] - 2026-01-10

### Added

- Checksum verification for downloaded assets.

## [1.0.0] - 2025-12-01

### Added

- Initial release.

---

[Unreleased]: https://github.com/Toraja/github-release-downloader/compare/v2.0.0...HEAD
[2.0.0]: https://github.com/Toraja/github-release-downloader/compare/v1.1.0...v2.0.0
[1.1.0]: https://github.com/Toraja/github-release-downloader/compare/v1.0.0...v1.1.0
[1.0.0]: https://github.com/Toraja/github-release-downloader/releases/tag/v1.0.0
```

## Review Checklist

- `## [Unreleased]` section present at the top.
- Versions in reverse chronological order with ISO 8601 dates.
- Entries grouped under the standard headings; no empty headings.
- Every breaking change starts with `**Breaking:**`.
- A single `---` rule sits between the oldest release and the footnote.
- Reference-style links defined for `[Unreleased]` and every version.
- Entries describe user-facing impact, not commit noise.

## Common Mistakes

| Mistake | Fix |
| --- | --- |
| Dumping raw commit messages | Curate entries by user-facing impact. |
| Missing `## [Unreleased]` section | Always keep one at the top. |
| Breaking change without prefix | Prefix the entry with `**Breaking:**`. |
| Footnote glued to last release | Separate the oldest release from the footnote with `---`. |
| Ambiguous or non-ISO dates | Use `YYYY-MM-DD`. |
| Inline version links | Use reference-style links defined in the footnote. |
| Empty headings left in place | Omit headings with no entries. |

## Sources

- Keep a Changelog 1.1.0: https://keepachangelog.com/en/1.1.0/
- Semantic Versioning 2.0.0: https://semver.org/spec/v2.0.0.html
