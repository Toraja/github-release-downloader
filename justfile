default:
	@just --list --unsorted

tests:
	@cargo test

lint:
	@cargo clippy --all-targets --all-features -- --deny warnings

format:
	@cargo fmt --all

release:
	@cargo release

release-execute:
	@cargo release --execute

# For this project, the dist artifact is .github/workflows/release.yml
[doc("Generate (or update) the artifacts of dist based on dist-workspace.toml")]
dist-generate:
	@dist generate
