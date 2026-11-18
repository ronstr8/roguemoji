# Host public play on this server: `ssh roguemoji@host` and https://host/roguemoji/play/.
# Run from a checkout on the server: `make install`. Recipes use sudo unless run as root.
#
# Config files are installed only where missing. One that exists and differs is
# left alone and reported, since it may hold local edits; FORCE=1 replaces it.

ACCOUNT    := roguemoji
PASSWORD   ?= roguemoji
OPT        := /opt/roguemoji
SESSION    := $(OPT)/roguemoji-session
BANNER     := $(OPT)/banner.txt
SSHD_CONF  := /etc/ssh/sshd_config.d/50-roguemoji.conf
UNIT       := /etc/systemd/system/roguemoji-ttyd.service
# ttyd to run; empty means the one on PATH (see scripts/find-ttyd).
TTYD       ?=
TTYD_MIN   := 1.7
SSH_UNIT   ?= ssh
# The game to install: a release tarball, or else the `make static` build in this checkout.
TARBALL    ?=
GAME_BIN   ?= ../target/x86_64-unknown-linux-musl/release/roguemoji
GAME_DATA  ?= ../data
# Where `make update` downloads releases from, and which one: a version or `latest`.
REPO       := ronstr8/roguemoji
RELEASE    ?= latest
BUILD      := build

SUDO := $(if $(filter 0,$(shell id -u)),,sudo)
export SUDO FORCE TTYD

.DEFAULT_GOAL := help
.PHONY: help install game update session banner ensure-user sshd slice ensure-ttyd ttyd password check status uninstall clean

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## ' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[33m%-12s\033[0m %s\n", $$1, $$2}'

install: game ensure-user sshd slice ttyd ## Everything: game, account, sshd, caps and the ttyd service
	@echo
	@echo "Done. Add nginx/roguemoji.conf to your site's TLS server block, then: nginx -t && systemctl reload nginx"
	@echo "  (On Apache: add apache/roguemoji.conf to the TLS VirtualHost, then: apachectl configtest && systemctl reload apache2)"
	@echo "Check the sshd settings each account gets with: make check"

game: ## Install or update the game in /opt/roguemoji (TARBALL=..., or the make static build)
	scripts/install-game $(OPT) $(if $(TARBALL),$(TARBALL),$(GAME_BIN) $(GAME_DATA))

update: ## Download a release (RELEASE=x.y.z, default latest), check its sha256, and install it
	@tarball=$$(scripts/fetch-release $(REPO) $(RELEASE) $(BUILD)) && scripts/install-game $(OPT) "$$tarball"

session: ## Install the wrapper that is the only thing players can run
	scripts/place roguemoji-session $(SESSION) 755

banner: $(BUILD)/banner.txt ## Install the SSH login banner, which shows PASSWORD
	scripts/place $< $(BANNER) 644

# Rebuilt every time, so a different PASSWORD always shows up as a difference.
$(BUILD)/banner.txt: sshd/banner.txt FORCE_REBUILD
	@mkdir -p $(BUILD)
	@sed 's/@PASSWORD@/$(PASSWORD)/' $< > $@

.PHONY: FORCE_REBUILD
FORCE_REBUILD:

ensure-user: session ## Create the roguemoji account if it doesn't exist (password PASSWORD)
	@if id -u $(ACCOUNT) >/dev/null 2>&1; then \
		shell=$$(getent passwd $(ACCOUNT) | cut -d: -f7); \
		[ "$$shell" = $(SESSION) ] || echo "warning: $(ACCOUNT)'s shell is $$shell, not $(SESSION)" >&2; \
	else \
		$(SUDO) useradd --system --create-home --home-dir /var/lib/$(ACCOUNT) --shell $(SESSION) $(ACCOUNT) && \
		echo '$(ACCOUNT):$(PASSWORD)' | $(SUDO) chpasswd && \
		echo "created    account $(ACCOUNT)"; \
	fi

sshd: session banner ensure-user ## Install the sshd drop-in, check it, and reload sshd
	scripts/place sshd/50-roguemoji.conf $(SSHD_CONF) 644
	@$(SUDO) sshd -t || { echo "sshd -t failed: fix the config before sshd restarts. Not reloading." >&2; exit 1; }
	$(SUDO) systemctl reload $(SSH_UNIT)

slice: ensure-user ## Cap memory, tasks and CPU for all SSH players together
	@uid=$$(id -u $(ACCOUNT)) && mkdir -p $(BUILD) && \
		sed "s/<UID>/$$uid/" systemd/user-slice.conf > $(BUILD)/user-slice.conf && \
		scripts/place $(BUILD)/user-slice.conf /etc/systemd/system/user-$$uid.slice.d/50-roguemoji.conf 644
	$(SUDO) systemctl daemon-reload

ensure-ttyd: ## Find ttyd (TTYD=path, or on PATH), apt-install it where the distro has it, and check it's new enough
	@scripts/find-ttyd $(TTYD_MIN) >/dev/null

ttyd: session ensure-user ## Install and start the browser service (restart it yourself after a unit change)
	@path=$$(scripts/find-ttyd $(TTYD_MIN)) && mkdir -p $(BUILD) && \
		sed "s|@TTYD@|$$path|" systemd/roguemoji-ttyd.service > $(BUILD)/roguemoji-ttyd.service
	scripts/place $(BUILD)/roguemoji-ttyd.service $(UNIT) 644
	$(SUDO) systemctl daemon-reload
	$(SUDO) systemctl enable --now roguemoji-ttyd

password: ## Set the account's password to PASSWORD and update the banner to match
	echo '$(ACCOUNT):$(PASSWORD)' | $(SUDO) chpasswd
	$(MAKE) banner FORCE=1

check: ## Show the sshd settings the roguemoji account and root really get
	@for u in $(ACCOUNT) root; do echo "== $$u"; \
		$(SUDO) sshd -T -C user=$$u,host=x,addr=203.0.113.1 | grep -Ei '^(forcecommand|passwordauthentication|disableforwarding|banner) '; \
	done

status: ## Show the ttyd service and how many players are connected
	@systemctl --no-pager status roguemoji-ttyd | head -5 || true
	@echo "games running: $$(pgrep -cxu $(ACCOUNT) roguemoji || true)"

uninstall: ## Stop the service and remove the sshd, systemd and slice files (keeps the account and /opt/roguemoji)
	-$(SUDO) systemctl disable --now roguemoji-ttyd
	$(SUDO) rm -f $(UNIT) $(SSHD_CONF)
	-uid=$$(id -u $(ACCOUNT) 2>/dev/null) && $(SUDO) rm -f /etc/systemd/system/user-$$uid.slice.d/50-roguemoji.conf
	$(SUDO) systemctl daemon-reload
	$(SUDO) systemctl reload $(SSH_UNIT)
	@echo "Kept account $(ACCOUNT) and $(OPT). Remove them with: userdel -r $(ACCOUNT); rm -rf $(OPT)"

clean: ## Remove generated files
	rm -rf $(BUILD)
