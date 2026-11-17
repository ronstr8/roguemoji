# roguemoji — common development targets.
# Pass a seed to play/run a reproducible world: `make run SEED=42`

CARGO   ?= cargo
BIN     := roguemoji
SEED    ?=
RUNARGS := $(if $(SEED),-- --seed $(SEED),)
PREFIX  ?= $(HOME)/.local
MUSL    := x86_64-unknown-linux-musl
WIN     := x86_64-pc-windows-gnu
VERSION := $(shell sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
TAG     := v$(VERSION)
DIST    := $(BIN)-$(VERSION)-x86_64-linux
WDIST   := $(BIN)-$(VERSION)-x86_64-windows
REPO_URL := https://github.com/ronstr8/roguemoji
# Added to every release's notes, after the changelog section. No single quotes.
DOWNLOAD_NOTE := **Download:** the `.tar.gz` runs on any 64-bit x86 Linux; the `.zip` runs on 64-bit Windows (play it in Windows Terminal, since the old console window draws emoji badly). Both include the stock `data/` folder beside the binary. Edit those files to mod the game, no rebuild needed (see `data/README.md`). When you upgrade, replace `data/` along with the binary.

.DEFAULT_GOAL := help
.PHONY: help build release static windows dist publish run debug test check lint fmt fmt-check doc clean install uninstall webplay ci

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

windows: ## Static, stripped Windows exe (MinGW) at target/$(WIN)/release/$(BIN).exe
	rustup target add $(WIN)
	RUSTFLAGS="-C strip=symbols -C target-feature=+crt-static" $(CARGO) build --release --target $(WIN)
	@ls -lh target/$(WIN)/release/$(BIN).exe

dist: static windows ## Package the Linux tarball and Windows zip, each with README, CHANGELOG and live data/
	rm -rf dist/$(DIST) && mkdir -p dist/$(DIST)
	cp -f target/$(MUSL)/release/$(BIN) README.md CHANGELOG.md dist/$(DIST)/
	cp -rf data dist/$(DIST)/data
	tar -C dist -czf dist/$(DIST).tar.gz $(DIST)
	cd dist && sha256sum $(DIST).tar.gz > $(DIST).tar.gz.sha256
	rm -rf dist/$(WDIST) dist/$(WDIST).zip && mkdir -p dist/$(WDIST)
	cp -f target/$(WIN)/release/$(BIN).exe README.md CHANGELOG.md dist/$(WDIST)/
	cp -rf data dist/$(WDIST)/data
	cd dist && zip -qr $(WDIST).zip $(WDIST) && sha256sum $(WDIST).zip > $(WDIST).zip.sha256
	awk -v version=$(VERSION) -f scripts/release-notes.awk CHANGELOG.md > dist/notes-$(TAG).md
	printf '\n%s\n' '$(DOWNLOAD_NOTE)' >> dist/notes-$(TAG).md
	printf '\nFull changelog: %s/blob/$(TAG)/CHANGELOG.md\n' "$(REPO_URL)" >> dist/notes-$(TAG).md
	@ls -lh dist/$(DIST).tar.gz dist/$(WDIST).zip

publish: ## Create the GitHub release for the tagged HEAD, with the Linux tarball and Windows zip
	@git diff --quiet HEAD && test -z "$$(git status --porcelain)" || { echo "publish: working tree is not clean"; exit 1; }
	@test "$$(git describe --exact-match --tags HEAD 2>/dev/null)" = "$(TAG)" || { echo "publish: HEAD is not tagged $(TAG) (Cargo.toml says $(VERSION))"; exit 1; }
	@git ls-remote --exit-code --tags origin refs/tags/$(TAG) >/dev/null || { echo "publish: $(TAG) is not pushed to origin"; exit 1; }
	@grep -q '^## \[$(VERSION)\]' CHANGELOG.md || { echo "publish: CHANGELOG.md has no [$(VERSION)] section"; exit 1; }
	@! gh release view $(TAG) >/dev/null 2>&1 || { echo "publish: $(TAG) is already released"; exit 1; }
	$(MAKE) ci dist
	gh release create $(TAG) --verify-tag --title "$(BIN) $(VERSION)" \
		--notes-file dist/notes-$(TAG).md dist/$(DIST).tar.gz dist/$(DIST).tar.gz.sha256 \
		dist/$(WDIST).zip dist/$(WDIST).zip.sha256

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

webplay: static ## Set up public SSH and browser play on this server (see deploy/README.md)
	$(MAKE) -C deploy install

uninstall: ## Remove from PREFIX/bin
	rm -f $(PREFIX)/bin/$(BIN)

clean: ## Remove build artifacts
	$(CARGO) clean
