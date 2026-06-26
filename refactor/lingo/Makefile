.PHONY: run build install test fmt clippy check clean

CARGO ?= cargo
PACKAGE ?= lingo-cli
BIN ?= lingo
PROFILE ?= dev
ARGS ?= --help
INSTALL_ROOT ?=
INSTALL_FLAGS ?= --locked
OFFLINE ?= 0

ifeq ($(PROFILE),release)
PROFILE_FLAG := --release
else
PROFILE_FLAG :=
endif

ifneq ($(INSTALL_ROOT),)
INSTALL_ROOT_FLAG := --root $(INSTALL_ROOT)
else
INSTALL_ROOT_FLAG :=
endif

ifeq ($(OFFLINE),1)
OFFLINE_FLAG := --offline
else
OFFLINE_FLAG :=
endif

run:
	$(CARGO) run -p $(PACKAGE) --bin $(BIN) -- $(ARGS)

build:
	$(CARGO) build -p $(PACKAGE) --bin $(BIN) $(PROFILE_FLAG)

install:
	$(CARGO) install --path crates/lingo-cli --bin $(BIN) $(INSTALL_FLAGS) $(OFFLINE_FLAG) $(INSTALL_ROOT_FLAG)

test:
	$(CARGO) test --workspace --all-targets

fmt:
	$(CARGO) fmt --all -- --check

clippy:
	$(CARGO) clippy --workspace --all-targets -- -D warnings

check: fmt test clippy

clean:
	$(CARGO) clean
