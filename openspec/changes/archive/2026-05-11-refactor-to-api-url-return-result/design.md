## Context

`to_api_url` in `src/main.rs` currently returns `Result<String, String>`. The string is a valid URL, but callers must re-parse it if they need URL-typed operations. Since `url::Url` is already a dependency, returning it directly is both safer and more ergonomic.

## Goals / Non-Goals

**Goals:**
- Change `to_api_url` return type to `Result<url::Url, String>`
- Update `main()` call site to display the URL via `.to_string()`
- Update unit tests to compare against `url::Url` values instead of strings

**Non-Goals:**
- Changing error handling strategy or error type
- Changing observable CLI behavior or output format
- Introducing new dependencies

## Decisions

### Use `Url::parse` to construct the return value

The API URL string `"https://api.github.com/repos/{}/{}/releases/latest"` is always well-formed when owner/repo are valid path segments. Using `Url::parse(...).expect(...)` (or `unwrap()`) at the construction point is appropriate — a panic here would indicate a programmer error, not a user input error. Alternatively, the error could be propagated as `String` via `.map_err(|e| e.to_string())`, which is slightly more defensive and avoids the `expect` call.

**Decision**: Use `.map_err(|e| e.to_string())` and return the error via `?` for consistency with the rest of the function's error handling pattern.

## Risks / Trade-offs

- [Minimal risk]: This is a purely internal refactor with no behavior change. Tests cover all meaningful cases.
- [Test assertions change]: Tests comparing `Ok("https://...")` must be updated to compare `Ok(Url::parse("https://...").unwrap())`. This is straightforward.
