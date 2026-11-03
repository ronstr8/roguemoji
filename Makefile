# roguemoji — common development targets.
# Pass a seed to play/run a reproducible world: `make run SEED=42`

CARGO   ?= cargo
BIN     := roguemoji
SEED    ?=
RUNARGS := $(if $(SEED),-- --seed $(SEED),)
PREFIX  ?= $(HOME)/.local
MUSL    := x86_64-unknown-linux-musl

.DEFAULT_GOAL := help
.PHONY: help build release static run debug test check lint fmt fmt-check doc clean install uninstall ci

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## ' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[33m%-10s\033[0m %s\n", $$1, $$2}'

build: ## Debug build
	$(CARGO) build

release: ## Optimized build
	$(CARGO) build --release

static: ## Fully static, stripped Linux binary (musl) at target/$(MUSL)/release/$(BIN)
	rustup target add $(MUSL)
	RUSTFLAGS="-C strip=symbols" $(CARGO) build --release --target $(MUSL)
	@ls -lh target/$(MUSL)/release/$(BIN)

run: ## Play (optimized). Optional: SEED=n
	$(CARGO) run --release $(RUNARGS)

debug: ## Play the debug build. Optional: SEED=n
	$(CARGO) run $(RUNARGS)

test: ## Run unit and soak tests (optimized for speed)
	$(CARGO) test --release

check: ## Type-check without building
	$(CARGO) check --all-targets

lint: ## Clippy, warnings as errors
	$(CARGO) clippy --all-targets -- -D warnings

fmt: ## Format the code
	$(CARGO) fmt

fmt-check: ## Verify formatting
	$(CARGO) fmt -- --check

doc: ## Build and open API docs
	$(CARGO) doc --no-deps --document-private-items --open

ci: fmt-check lint test ## Everything CI would run

install: release ## Install to PREFIX/bin (default ~/.local/bin)
	install -Dm755 target/release/$(BIN) $(PREFIX)/bin/$(BIN)

uninstall: ## Remove from PREFIX/bin
	rm -f $(PREFIX)/bin/$(BIN)

clean: ## Remove build artifacts
	$(CARGO) clean
