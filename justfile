default:
	@just --list --unsorted

build:
	@cargo build --all-features

test:
	@cargo nextest run

test-coverage:
	@cargo llvm-cov nextest --all-features --fail-under-lines 75

lint:
	@cargo clippy --all-targets --all-features -- --deny warnings

format:
	@cargo fmt --all

release level="":
	@cargo release {{level}}

release-execute level="":
	@cargo release --execute {{level}}

# For this project, the dist artifact is .github/workflows/release.yml
[doc("Generate (or update) the artifacts of dist based on dist-workspace.toml")]
dist-generate:
	@dist generate
