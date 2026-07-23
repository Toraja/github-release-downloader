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
