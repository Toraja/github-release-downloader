## 1. Rename the binary

- [x] 1.1 Add a `[[bin]]` section to `Cargo.toml` with `name = "ghrls"` and `path = "src/main.rs"`
- [x] 1.2 Build to confirm the produced binary is named `ghrls` and regenerate `Cargo.lock`

## 2. Update references to the binary name

- [x] 2.1 Update the `User-Agent` header strings in `src/github.rs` (lines 48 and 92) to `ghrls`
- [x] 2.2 Update `README.md` usage examples and `--help` references to use `ghrls`

## 3. Verify

- [x] 3.1 Run `just test`
- [x] 3.2 Run `just lint`
- [x] 3.3 Run `just format`
