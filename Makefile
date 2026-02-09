.PHONY: build run test clean bump

ifneq (,$(wildcard .env))
    include .env
    export
endif

COMMIT_HASH = $(shell git rev-parse --short HEAD)
CARGO_MANIFEST_PATH = Cargo.toml

# Use `make build ARGS="--release"` for release mode or other arguments.
build:
	cargo clippy --manifest-path $(CARGO_MANIFEST_PATH) -- -D warnings --no-deps
	COMMIT_HASH=$(COMMIT_HASH) \
	cargo build \
		--manifest-path $(CARGO_MANIFEST_PATH) \
		$(ARGS)

# Use `make run ARGS="--release"` for release mode or other arguments.
run:
	COMMIT_HASH=$(COMMIT_HASH) \
	RUST_BACKTRACE=1 \
	cargo run \
		--manifest-path $(CARGO_MANIFEST_PATH) \
		$(ARGS)

# Use `make test ARGS="--release"` for release mode or other arguments.
test:
	COMMIT_HASH=$(COMMIT_HASH) \
	RUST_BACKTRACE=1 \
	cargo test \
		--manifest-path $(CARGO_MANIFEST_PATH) \
		$(ARGS) \
		-- --no-capture

check:
	cargo clippy --manifest-path $(CARGO_MANIFEST_PATH) -- -D warnings --no-deps

clean:
	cargo clean --manifest-path $(CARGO_MANIFEST_PATH)