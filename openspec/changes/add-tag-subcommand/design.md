## Context

The CLI already models top-level operations as Clap subcommands and has shared functions that convert a GitHub repository URL to the latest-release API endpoint and fetch its JSON response. The deserialized release model currently retains only assets, while GitHub's response also includes `tag_name`. See `proposal.md` for motivation and `specs/release-tag/spec.md` for observable behavior.

The tag output is intended for shell command substitution and other automation, so it must contain no labels or progress text. GitHub tags are not constrained to semantic versions and must remain opaque strings.

## Goals / Non-Goals

**Goals:**

- Reuse the same latest-release request, URL validation, authentication, and error mapping as asset downloads.
- Keep tag retrieval separate from asset selection and download I/O.
- Make prefix removal a small, independently testable literal transformation.
- Preserve exact automation-friendly stdout behavior.

**Non-Goals:**

- Parse, validate, compare, or normalize semantic versions.
- Support selecting releases other than GitHub's latest published release.
- Add a generic output templating or JSON mode.
- Change existing download behavior.

## Decisions

### Add `tag_name` to the shared release model

**Decision:** Extend the existing release response type with a string field mapped from GitHub's `tag_name`, while retaining its assets field.

**Rationale:** Both subcommands call the same endpoint and share request semantics. One response model keeps authentication, HTTP handling, and JSON error behavior consistent and avoids duplicate network code.

**Alternative considered:** Introduce a tag-only response type and generic fetch helper. Rejected because it adds type and deserialization plumbing without reducing the response body transferred by GitHub or providing meaningful separation for this small API model.

### Model `tag` as a dedicated subcommand argument type

**Decision:** Add a `Tag` command variant containing a repository `Url` and optional `String` for `--strip-prefix`, mirroring the existing typed `Download` subcommand structure.

**Rationale:** Parsing the URL through Clap preserves early URL syntax validation, and a dedicated type keeps tag-only options out of other commands. Clap will also include the new command and option in generated shell completions automatically.

**Alternative considered:** Add a flag to `download` that switches its output from an asset to a tag. Rejected because tag retrieval does not need an asset pattern and is a distinct operation with different output behavior.

### Transform only the printed view of the tag

**Decision:** Keep the fetched tag unchanged in the release model. At command dispatch, derive the printed value with one standard leading-prefix removal when `--strip-prefix` is present, falling back to the original tag when it does not match.

**Rationale:** This directly provides literal, case-sensitive, leading-only behavior and removes at most one occurrence. Applying the transformation at the output boundary avoids altering release metadata that could be reused elsewhere.

**Alternative considered:** Use regular expressions or semantic-version parsing. Rejected because both introduce semantics beyond literal prefix removal and would complicate arbitrary valid GitHub tags.

### Print the value with a single newline

**Decision:** Write only the transformed tag followed by a newline to stdout. If stripping consumes the complete tag, print a newline and return success.

**Rationale:** Plain line output composes naturally with shells and text-processing tools. Empty output is a valid result of the specified literal transformation, not an error.

**Alternative considered:** Reject an empty transformed tag. Rejected because it would make prefix stripping context-sensitive and imply version validation that the command intentionally does not perform.

## Risks / Trade-offs

- [Adding required `tag_name` deserialization can make download fail if a nonconforming API response omits that field] -> This is acceptable because GitHub's release schema defines `tag_name`, and a malformed release response should remain a parse error.
- [A successful newline-only result can be mistaken for missing data by callers] -> Document and test that complete-prefix removal is valid literal behavior.
- [The command reports GitHub's latest release, not necessarily the repository's highest semantic version or newest tag] -> Name and document the behavior in terms of the latest release endpoint and avoid version terminology.

## Migration Plan

No migration or compatibility handling is required. The new subcommand and response field are additive. Rollback consists of removing the command and the additional deserialized field.
