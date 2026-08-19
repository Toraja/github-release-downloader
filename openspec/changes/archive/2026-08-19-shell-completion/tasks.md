## 1. Add dependency

- [x] 1.1 Add `clap_complete` (matching clap v4) to `[dependencies]` in `Cargo.toml`

## 2. Restructure the CLI parser

- [x] 2.1 Rename the existing `Args` struct to a `Download` args struct (keep all current fields and the `try_validate`/`validate` logic)
- [x] 2.2 Add a top-level `Cli` parser with a `#[command(subcommand)]` enum containing `Download(...)` and `Completion { shell: clap_complete::Shell }` variants
- [x] 2.3 Ensure `clap_complete::Shell` is used as the `completion` positional argument so clap validates shell names and lists supported shells on error

## 3. Wire up completion generation

- [x] 3.1 In `run()`, match on the subcommand: for `Completion { shell }`, call `clap_complete::generate()` with `Cli::command()` and the binary name (`ghrls`) writing to stdout, then return `Ok(())` before any network or file I/O
- [x] 3.2 For `Download`, run the existing validate + fetch + download/extract flow unchanged

## 4. Update tests

- [x] 4.1 Update existing `Args::try_parse_from([...])` tests to parse through the new subcommand structure (prefix positional invocations with the `download` subcommand)
- [x] 4.2 Add a test that `ghrls completion bash` parses successfully and selects the `Completion` variant
- [x] 4.3 Add a test that an unrecognized shell name is rejected by clap

## 5. Documentation

- [x] 5.1 Update `README.md` usage to show the `download` and `completion` subcommands (e.g., `ghrls download <URL> <PATTERN>`, `ghrls completion <SHELL>`)

## 6. Verify

- [x] 6.1 Run `just test`
- [x] 6.2 Run `just lint`
- [x] 6.3 Run `just format`
