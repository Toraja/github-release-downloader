default:
	@just --list --unsorted

build:
	@cargo build --all-features

test:
	@cargo nextest run

test-coverage:
	@cargo llvm-cov nextest --all-features --fail-under-lines 80

[doc("Generate test coverage report in lcov format")]
test-coverage-lcov:
	@mkdir --parent target/coverage
	@cargo llvm-cov nextest --all-features --lcov --output-path target/coverage/lcov.info

lint:
	@cargo clippy --all-targets --all-features -- --deny warnings

format:
	@cargo fmt --all

release-dry-run level:
	@cargo release {{level}}

release level:
	@cargo release --execute --no-publish {{level}}

# For this project, the dist artifact is .github/workflows/release.yml
[doc("Generate (or update) the artifacts of dist based on dist-workspace.toml")]
dist-generate:
	@dist generate
