.PHONY: test fmt clippy check build release install uninstall

test:
	cargo test --workspace --all-targets
fmt:
	cargo fmt --all -- --check
clippy:
	cargo clippy --workspace --all-targets -- -D warnings
check: fmt test clippy
build:
	cargo build --workspace
release:
	cargo build --workspace --release
install:
	cargo install --path crates/lingo-cli --force
uninstall:
	cargo uninstall lingo-cli
