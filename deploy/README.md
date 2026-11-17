# Hosting public play

How to let anyone play roguemoji on your server, in two ways:

- **SSH:** `ssh roguemoji@your.host`, with a password printed in the login banner.
- **Browser:** `https://your.host/play/`, through [ttyd](https://github.com/tsl0922/ttyd)
  behind your web server.

Each connection gets its own game in a fresh random world. The game writes no
files and starts no programs, so the only things to lock down are SSH and the
resources players can use.

| File | Goes to | What it does |
|---|---|---|
| `roguemoji-session` | `/opt/roguemoji/` | The one program players can run. Clears `ROGUEMOJI_DATA`, sets a UTF-8 locale, and starts the game |
| `sshd/50-roguemoji.conf` | `/etc/ssh/sshd_config.d/` | Password login for the `roguemoji` account only, which always runs the game. No forwarding or tunnels |
| `sshd/banner.txt` | `/opt/roguemoji/` | Shown before the password prompt. Tells players the password |
| `systemd/roguemoji-ttyd.service` | `/etc/systemd/system/` | ttyd on loopback port 7681, sandboxed, with shared memory, task and CPU caps |
| `systemd/user-slice.conf` | `/etc/systemd/system/user-<UID>.slice.d/50-roguemoji.conf` | The same caps for SSH players |
| `nginx/roguemoji.conf` | your site's TLS `server { }` block | Proxies `/play/` and its websocket to ttyd |

## Setup

Run these as root on the server. They assume Debian or Ubuntu with OpenSSH 8.2
or newer (for `sshd_config.d`) and nginx.

1. **Install the game.** Unpack a Linux release (or the output of `make static`)
   into `/opt/roguemoji`, so that the binary and `data/` sit side by side. Then
   add this folder's wrapper and banner:

   ```sh
   mkdir -p /opt/roguemoji
   tar -C /opt/roguemoji --strip-components=1 -xzf roguemoji-X.Y.Z-x86_64-linux.tar.gz
   install -m 755 deploy/roguemoji-session /opt/roguemoji/
   install -m 644 deploy/sshd/banner.txt /opt/roguemoji/
   chown -R root:root /opt/roguemoji
   ```

   Everything there stays owned by root. Players can't change the game data.

2. **Create the account.** Its login shell is the wrapper, so even `su` lands in
   the game. Set the password the banner prints (`roguemoji` unless you edit it):

   ```sh
   useradd --system --create-home --shell /opt/roguemoji/roguemoji-session roguemoji
   passwd roguemoji
   ```

3. **SSH.** Install the config, check it, and reload:

   ```sh
   install -m 644 deploy/sshd/50-roguemoji.conf /etc/ssh/sshd_config.d/
   sshd -t
   sshd -T -C user=roguemoji,host=x,addr=203.0.113.1 | grep -Ei 'forcecommand|passwordauth|disableforwarding'
   sshd -T -C user=root,host=x,addr=203.0.113.1 | grep -Ei 'forcecommand|passwordauth'
   systemctl reload ssh
   ```

   The two `sshd -T` lines show the settings each account really gets. The
   roguemoji account should have the `forcecommand` and password login. Root
   (and you) should keep your usual settings, with no `forcecommand`. Keep your
   current SSH session open until you've logged in again from a new one.

   With many players, raise sshd's global `MaxStartups` (default `10:30:100`)
   so that a burst of logins isn't refused.

4. **Cap SSH players.** Copy the slice drop-in and fill in the account's UID:

   ```sh
   uid=$(id -u roguemoji)
   mkdir -p /etc/systemd/system/user-$uid.slice.d
   sed "s/<UID>/$uid/" deploy/systemd/user-slice.conf > /etc/systemd/system/user-$uid.slice.d/50-roguemoji.conf
   systemctl daemon-reload
   ```

5. **Browser.** Install ttyd 1.7 or newer (`apt install ttyd`, or a static
   binary from its releases page; the unit expects `/usr/bin/ttyd`). Then start
   the service:

   ```sh
   install -m 644 deploy/systemd/roguemoji-ttyd.service /etc/systemd/system/
   systemctl daemon-reload
   systemctl enable --now roguemoji-ttyd
   ```

   Paste `nginx/roguemoji.conf` into your site's TLS `server { }` block,
   then run `nginx -t && systemctl reload nginx`. ttyd listens only on
   loopback, so nginx is the only way in from outside.

   On Apache, the same proxy is `ProxyPass /play/ http://127.0.0.1:7681/play/ upgrade=websocket`,
   which needs `mod_proxy_http` and Apache 2.4.47 or newer.

## Updating

Unpack the new release over `/opt/roguemoji`, replacing `data/` too. Running
games keep the old version until their players quit, and new connections get
the new one. Nothing needs restarting.

## Notes

- **Emoji width** depends on the player's terminal. The browser terminal
  (xterm.js) gets it right. Old terminals and some fonts draw emoji one cell
  wide, which misaligns the map. The game asks for at least 80×24 itself.
- **Idle players** stay connected for as long as their client does. Dead
  connections are dropped (SSH after about three minutes; ttyd pings its
  websocket). `--max-clients` in the ttyd unit limits how many browser players
  can connect at once.
- **No saves yet:** a dropped connection ends the run.
