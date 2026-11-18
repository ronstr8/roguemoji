# Hosting public play

How to let anyone play roguemoji on your server, in two ways:

- **SSH:** `ssh roguemoji@your.host`, with a password printed in the login banner.
- **Browser:** `https://your.host/roguemoji/play/`, through [ttyd](https://github.com/tsl0922/ttyd)
  behind your web server.

Each connection gets its own game in a fresh random world. The game writes no
files and starts no programs, so the only things to lock down are SSH and the
resources players can use.

| File | Goes to | What it does |
|---|---|---|
| `roguemoji-session` | `/opt/roguemoji/` | The one program players can run. Clears `ROGUEMOJI_DATA`, sets a UTF-8 locale, and starts the game |
| `sshd/50-roguemoji.conf` | `/etc/ssh/sshd_config.d/` | Password login for the `roguemoji` account only, which always runs the game. No forwarding or tunnels |
| `sshd/banner.txt` | `/opt/roguemoji/` | Shown before the password prompt. Tells players the password (`@PASSWORD@` is filled in on install) |
| `systemd/roguemoji-ttyd.service` | `/etc/systemd/system/` | ttyd (path filled in for `@TTYD@` on install) on loopback port 7681, sandboxed, with shared memory, task and CPU caps |
| `systemd/user-slice.conf` | `/etc/systemd/system/user-<UID>.slice.d/50-roguemoji.conf` | The same caps for SSH players |
| `Makefile`, `scripts/` | | Install all of the above (see Setup) |
| `nginx/roguemoji.conf` | your site's TLS `server { }` block | Proxies `/roguemoji/play/` and its websocket to ttyd |
| `apache/roguemoji.conf` | your site's TLS `<VirtualHost *:443>` block | The same proxy for Apache instead of nginx |

## Setup

On the server, from a checkout of this repository (Debian or Ubuntu, OpenSSH
8.2 or newer for `sshd_config.d`, and nginx):

```sh
make static                # or skip this and pass TARBALL= below
make -C deploy install     # or TARBALL=roguemoji-X.Y.Z-x86_64-linux.tar.gz
make -C deploy check
```

`make webplay` at the top level does the first two steps together. The
recipes use `sudo` unless you run them as root. `make -C deploy` alone lists
every target. `install` runs these in order:

| Target | What it does |
|---|---|
| `game` | Puts the binary and `data/` in `/opt/roguemoji`, owned by root, so players can't change the game data |
| `ensure-user` | Creates the `roguemoji` system account if it's missing. Its home is `/var/lib/roguemoji`, its password is `PASSWORD` (default `roguemoji`), and its login shell is the wrapper, so even `su` lands in the game. It warns if an existing account has a different shell |
| `sshd` | Installs the wrapper, banner and sshd drop-in, runs `sshd -t`, and reloads sshd only if that passes. Set `SSH_UNIT=sshd` on distros that name the service that way |
| `slice` | Fills in the account's UID and installs the cap for SSH players |
| `ttyd` | Finds ttyd (`ensure-ttyd`): `TTYD=path` if given, else the one on `PATH`. If there's none it installs the apt package where the distro has one; Debian 13 (trixie) doesn't, so it prints the commands to put the static binary from [ttyd's releases](https://github.com/tsl0922/ttyd/releases) in `/usr/local/bin` and stops. It also stops if ttyd is older than 1.7. Then it installs and starts the service, with that ttyd's path in `ExecStart` |

**Config files are installed only where they're missing.** If one already
exists and differs, it's left alone and reported, since you may have edited it
on the server. Diff it against this folder, then run with `FORCE=1` to replace
it; the old file is kept beside it as `.orig`. Unit changes need
`systemctl restart roguemoji-ttyd`, which ends games in progress.

**After `install`:**

- `make -C deploy check` shows the sshd settings each account really gets. The
  roguemoji account should have the `forcecommand` and password login. Root
  (and you) should keep your usual settings, with no `forcecommand`. Keep your
  current SSH session open until you've logged in again from a new one.
- Paste `nginx/roguemoji.conf` into your site's TLS `server { }` block, then
  run `nginx -t && systemctl reload nginx`. ttyd listens only on loopback, so
  nginx is the only way in from outside. On Apache, paste
  `apache/roguemoji.conf` into the TLS `<VirtualHost *:443>` block instead
  (above any catch-all `ProxyPass`), enable `proxy` and `proxy_http`, then run
  `apachectl configtest && systemctl reload apache2`. It needs Apache 2.4.47
  or newer.
- With many players, raise sshd's global `MaxStartups` (default `10:30:100`)
  so that a burst of logins isn't refused.

Other targets: `password PASSWORD=...` changes the password and the banner
together, `status` shows the service and how many games are running, and
`uninstall` removes the service and config files but keeps the account and
`/opt/roguemoji`.

## Updating

`make -C deploy game` (after `make static`, or with `TARBALL=`) replaces the
binary and `data/`. Running games keep what they loaded until their players
quit, and new connections get the new version. Nothing needs restarting.

## Notes

- **Emoji width** depends on the player's terminal. The browser terminal
  (xterm.js) gets it right. Old terminals and some fonts draw emoji one cell
  wide, which misaligns the map. The game asks for at least 80×24 itself.
- **Idle players** stay connected for as long as their client does. Dead
  connections are dropped (SSH after about three minutes; ttyd pings its
  websocket). `--max-clients` in the ttyd unit limits how many browser players
  can connect at once.
- **No saves yet:** a dropped connection ends the run.
