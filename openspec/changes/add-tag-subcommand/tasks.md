## 1. Release Metadata

- [x] 1.1 Extend the shared GitHub release response model with `tag_name` and verify deserialization tests cover release tags alongside assets.

## 2. Tag Command

- [x] 2.1 Add the typed `tag <URL> [--strip-prefix <PREFIX>]` CLI arguments and verify Clap parsing tests cover the URL and optional prefix.
- [x] 2.2 Implement literal, case-sensitive, leading-only prefix removal and verify unit tests cover matching, absent, repeated, case-mismatched, non-semantic, and complete-tag prefixes.
- [x] 2.3 Dispatch the `tag` command through the existing latest-release request and print only the transformed tag plus one newline; verify command behavior does not enter asset selection or download paths.

## 3. Documentation And Verification

- [x] 3.1 Document the `tag` subcommand and `--strip-prefix` behavior in the README and changelog, including an automation-oriented usage example, and verify the documented syntax matches generated CLI help.
- [x] 3.2 Run `just test`, `just lint`, and `just format` and verify all checks complete successfully.
