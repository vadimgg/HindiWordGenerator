.PHONY: build test fmt clippy check run doctor plan generate audio quality viewer viewer-check viewer-build export install clean

SOURCE ?= Complete Hindi
TOPIC ?= Chapter 02
MAX_BATCHES ?= 1

build:
	cargo build

test:
	cargo test

fmt:
	cargo fmt --check

clippy:
	cargo clippy --all-targets --all-features -- -D warnings

check: fmt test clippy viewer-check

run: doctor

doctor:
	cargo run -- doctor

plan:
	cargo run -- sentences plan --max-batches $(MAX_BATCHES)

generate:
	cargo run -- sentences generate --max-batches $(MAX_BATCHES)

audio:
	cargo run -- sentences audio

quality:
	cargo run -- sentences review-output

viewer:
	cargo run -- viewer

viewer-check:
	cd viewer && npm run check

viewer-build:
	cd viewer && npm run build

export:
	cargo run -- export --source "$(SOURCE)" --topic "$(TOPIC)"

install:
	cargo install --path .

clean:
	cargo clean
