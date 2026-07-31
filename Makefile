# ----- Makefile -----
BRANCH := $(shell git branch --show-current)
REMOTES := $(shell git remote)

# ----- Targets -----
.PHONY: all build check release install uninstall fmt fmt-check clippy test tests clean push push-lease

# ----- Default -----
all: build

# ----- Builds (Linux) -----
build:
	cargo build --locked

check: fmt-check clippy test

release: check
	cargo build --release --locked

install: release
	cargo install --path . --locked --force

uninstall:
	cargo uninstall vc-cli

# ----- Rust -----
fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

clippy:
	cargo clippy --workspace --all-targets --all-features --locked -- -D warnings

test:
	cargo test --workspace --locked

tests: test

# ----- Clean -----
clean:
	cargo clean

# ----- DEVELOPMENT (Push) -----
push:
	@echo "Push normal → branch: $(BRANCH)"
	@for remote in $(REMOTES); do \
		echo "  pushing to $$remote..."; \
		git push $$remote $(BRANCH); \
	done

push-lease:
	@echo "Push --force-with-lease → branch: $(BRANCH)"
	@for remote in $(REMOTES); do \
		echo "  pushing to $$remote..."; \
		git push --force-with-lease $$remote $(BRANCH); \
	done
