## Why

Users of automated workflows sometimes need the latest GitHub release tag without downloading an asset or assembling their own GitHub API and JSON-processing command. Providing this directly also supports repositories whose release tags optionally include a conventional prefix such as `v`.

## What Changes

- Add a `tag` subcommand that fetches the latest release for a GitHub repository and prints its tag to stdout.
- Add an optional `--strip-prefix <PREFIX>` argument that removes one literal, case-sensitive prefix from the start of the tag when present.
- Preserve the tag unchanged when the requested prefix is absent, and allow stripping to produce an empty result.
- Reuse the existing GitHub repository URL handling, latest-release lookup, authentication, and error behavior without downloading release assets.

## Capabilities

### New Capabilities

- `release-tag`: Retrieve and print the latest GitHub release tag, with optional literal prefix removal.

### Modified Capabilities

<!-- No existing capability requirements change. -->

## Impact

- The public CLI gains `ghrls tag <URL> [--strip-prefix <PREFIX>]`.
- GitHub release metadata parsing must include the API's `tag_name` field.
- CLI parsing, command dispatch, tests, shell completion output, and user documentation are affected.
- No new runtime dependency is expected.
