# Changelog

All notable changes to roguemoji are listed here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow
[Semantic Versioning](https://semver.org/). While the game is below 1.0, a minor
version can change rules, controls or data-file formats.

## [Unreleased]

## [0.9.0] - 2026-11-20

### Added

- Hidden caches. A 🪖 Military Crate (ruins) and a 🪧 Buried Cache (wastes) aren't
  drawn until you notice them: one Scavenging or Survival check when they first
  come into view, or walk into one to stumble onto it. Pick a crate's lock with
  Lockpicks or pry it with a crowbar on Brawn, choosing from a pop-up that shows
  your odds; dig up a cache with a shovel, at a cost in rest.
- A Lockpicking skill (Reflexes and Brains). The Scavenger starts at rank 1 with a
  set of 🪛 Lockpicks.
- Autovends sometimes stock a tool, most often a shovel, and the shovel and crowbar
  now have prices.
- Encounters glow on the map so they stand out from items: a slow pulse of
  green for hopeful ones, pale yellow for gamblers and red for horrors.

### Changed

- Harvest nodes (🌿 🚗 🧰 …) are data files in `data/harvest/`, so mods can add,
  change or rebalance them. Their numbers haven't changed, but a given seed now
  lays out a different world than it did in 0.8.0.
- The title screen, the sidebar and the SSH login banner spell the name r💀guem☢️ji.

## [0.8.0] - 2026-11-19

### Changed

- The way to the next world is a portal 🌀 instead of stairs 🔽. Walk into it and
  it takes you at once, with a half-second white flash.
- Levels are called phases now: "Phase 3" instead of "Depth 3" on the HUD,
  character sheet, log and death message.
- The README is now a short tour of the game, with a logo and mascot. Controls and
  rules moved to `docs/GUIDE.md` (included in downloads), and build, release and
  code notes to `docs/DEVELOPER.md`.

### Removed

- The `>` descend key. Portals take you without one.

## [0.7.0] - 2026-11-19

### Added

- Gambling. Wandering gamblers (🃏 Card Sharp, 💪 Arm Wrestler) offer one bet
  and move on; dens (🎲 Bones in the Ruins) take bets until the house purse runs
  dry. You stake crystals, so you're betting your healing. Some games are pure
  luck and others test Wit or Brawn. Encounter files take a `[wager]` table.
- Flavor markup for encounter text: `{ red | blue }` picks a phrase, `@tag`s keep linked
  choices in agreement, and the Friendly Stranger, Garden in the Rubble and Peaceful
  Sanctum now read a little differently each time.
- Cartridge files take an optional `short` name for tight tables.

### Changed

- Melee damage in item details is what you'd deal, with your Brawn bonus
  folded in: `4-7 (2-5, +2 Brawn)` instead of `2-5 (+Brawn)`.
- Melee weapon, gun and pack details show what you have equipped beside each
  stat, in green where your gear's number is better and red where it's worse.
  With nothing equipped, a weapon compares against your fists and a pack
  against carrying none.
- Gun details list their stats one per line under the description: Type,
  Barrel, Capacity, Skill, Accuracy, Reload Turns and Weight. Short terminals
  fold them into one wrapped line so the load table still fits.
- Melee weapons and armor get the same stats block: Skill, Damage,
  Accuracy, Penetration and Weight for melee; Armor, Dodge Penalty and
  Weight for armor.
- Food, drink and packs get a stats block too. Consumables list only what
  they do (Satiety, Hydration, Heal, Cures, Effects); packs list their
  Carry Capacity. Both list Weight.
- The load table's first column is headed "Load" instead of "Name".
- The Autovend window grows taller on big terminals.
- The load table names each load by type ("JHP", "Buck ×9") instead of its
  full label, since the gun's cartridge is already in its stats. Loads of
  another cartridge the gun can fire show that cartridge (".38 Spl"). The
  weight column is now "Gr" in plain grains, so names fit on an 80-column
  terminal.

### Fixed

- The sidebar colors each need by its own stage. A fine need is no longer
  shown red because the need beside it is ("Fed · Thirsty").
- Things lighter than a tenth of a pound show two decimals ("0.01 lb")
  instead of reading as weightless.

## [0.6.0] - 2026-11-18

### Added
- Rest. Every turn tires you, fighting twice as fast and heavy loads faster
  (×1.5 Burdened, ×2 Strained). Tired lowers Reflexes; Exhausted lowers
  Reflexes and Wit. At empty you collapse 💤 and sleep where you stand until
  you've recovered somewhat, unless something hits you.
- Press `z` to nap until fully rested. A monster coming into view or any key
  wakes you. Sleep heals 1 HP every 10 turns; hunger and thirst keep draining.
- Mods can give needs `fighting` and `burden` drain multipliers; `rest` takes
  a `sleep` table (`recover`, `wake_at`, `heal_every`).
- Hope, a fourth need, for your will to keep going. It fades slowly on its own.
  Downcast lowers Willpower; Despairing lowers it further, and Wit too, so
  crystals heal less and dread lands more often. It never costs HP.
- Encounters: scenes on the map you walk into. A pop-up tells the story, then
  they change your needs and sometimes hand you items. Ten to start: kindness
  (🤝 🔥 🌻 📻 💌), horrors you can steel yourself against with Willpower
  (😱 🧳 👀), and places that stay for you to return to (🧘 🪔).
- Mods can add encounters in `data/encounters/`.
- `make -C deploy update` updates a public-play server to the newest release
  (or `RELEASE=x.y.z`), checking the download against its sha256.

### Changed
- The sidebar shows needs two to a line (`Fed  ·  Hydrated`, `Rested  ·
  Hopeful`), with crystals below; the character sheet lists one need per line.
- Help scrolls when the terminal is too short to show it all (↑↓ or j/k,
  PgUp/PgDn, Home/End); the footer shows which lines you're seeing. `?` closes it.

## [0.5.0] - 2026-11-18

### Added
- The Autovend shows your load and burden under its tabs, and each item's
  weight: what selling it sheds, or what buying it adds.
- When you're over Light, the Autovend and Visitor Lockbox say how much to shed
  to reach the next lighter level, e.g. "shed 4.2 lb to be Light".
- `deploy/` holds what you need to host public play: a locked-down sshd config
  for `ssh roguemoji@host`, a sandboxed ttyd service for play in the browser,
  nginx and Apache snippets, and setup steps. `make webplay` builds the game and
  installs it all, creating the `roguemoji` account if needed.
- Thirst. It drains three times as fast as hunger: Thirsty lowers Brains and
  Wit, and Parched lowers them more and costs HP. The sidebar shows it beside
  hunger.
- Press `d` to drink from water underfoot or next to you, or drink Dirty Water
  from your pack. Either takes a Survival check, or you get sick.
- Boiled Water: craft it from Dirty Water to drink it safely.
- Drinks quench thirst: water, tea, broth, juice and soda. Moonshine dries you out.
- The inventory shows your Satiety and Hydration, so you can watch them rise as
  you eat and drink.
- Equipped gear is selectable in the inventory: its stats show in the detail
  panel beside its value and weight, for comparing with what's in your pack.
  Enter on an equipped item stows it in the pack.
- The Visitor Lockbox sorts both columns with `s` (name, weight or value, the
  same order as the inventory) and shows each item's value beside its weight.
- Mods can retune hunger and thirst in `data/needs/`, give consumables
  `hydration`, and make components drinkable with `drink`.

### Changed
- 20ga #3 Buck carries 470 grains of shot instead of 400, matching real
  loads. Its full-pattern damage rises from about 13.9 to 15.0.
- Brawn and Brains are abbreviated STR and INT on the title and character
  screens, instead of BRN and BRA.
- A gun's loads show as a table (Name, Wt, FPS, Rng, DMG, Pen) instead of
  lines that wrapped into each other, with the loaded one marked ▸. Muzzle
  energy in joules is no longer listed; damage is derived from it.
- Gun descriptions are shorter, so they wrap cleanly in the detail panel.
- Bottled Water quenches thirst instead of filling you up (no more satiety).
- `satiety` is optional in consumable files, defaulting to 0.
- The character sheet shows Satiety and Hydration out of 1000 with their
  current stage, replacing the old Satiety line.
- Burden and need stages sit in parentheses after their numbers, e.g.
  `Load 9.1 lb / 39.0 lb (Light)` and `Satiety 750/1000 (Fed)`.

## [0.4.0] - 2026-11-16

### Added
- Release downloads include the stock `data/` folder beside the binary, ready
  to edit: the game reads it at startup, so changes need no rebuild.
- `make dist` packages a release tarball, and `make publish` checks the release
  is ready and creates it on GitHub.
- A Windows build: each release includes a `.zip` with `roguemoji.exe` and
  `data/`. Play it in Windows Terminal; the old console window draws emoji
  badly. `make windows` cross-compiles it from Linux.

## [0.3.0] - 2026-11-15

### Added
- Shoot with `s`. If only one creature is in range, you fire at it. If several
  are, a list opens over the sidebar, nearest first, with each one's health,
  distance and hit %. Enter or `s` shoots.
- A shot that grazes or better flashes 💥 on the creature for 300ms.
- Every gun has a max range for each load: its effective range plus one tile
  per 400 fps of muzzle velocity. A snub .38 reaches 5 tiles; a .308 rifle
  reaches 25. Nothing past max range can be targeted.
- The inventory shows each load's effective and max range.
- The title screen shows the version, and `roguemoji --version` prints it.

### Changed
- Cycle stance moved from `s` to `S`.
- `f` targeting lists only creatures within the gun's max range.

## [0.2.0] - 2026-11-03

### Added
- Encumbrance. Everything has a weight, including worn gear and loaded rounds.
  Carrying more than you can manage makes you Burdened, Strained or Overloaded.
- Packs 🎒 that raise carry capacity, from the craftable Rag Satchel (+8 lb) to
  the Visitors' Grav Harness (+50 lb).
- Visitor lockboxes 🛅 that all open onto one shared 20-slot stash.
- Autovends buy items from your pack for half their shop price (Tab in the kiosk).
- The inventory lists what each item would sell for. `s` sorts the pack list by
  name, weight or value, and the Autovend's Sell tab uses the same order.
- The Proton Axe 🪓, a rare Visitor blade found from depth 6.
- All content is data: monsters, components, consumables, recipes, melee
  weapons, armor, packs, guns, cartridges, ammo and Autovend prices load from
  TOML files under `data/`. Drop-in files next to the executable or in
  `$ROGUEMOJI_DATA` add or override content, and startup reports bad files
  with their line numbers.
- `make static` builds a self-contained musl Linux binary.

### Changed
- Space waits and `5` picks up.
- Gun prices rank by overall quality (damage, capacity, accuracy, range and
  reload) instead of barrel length.

### Fixed
- The Autovend's Buy and Sell tabs were drawn at the wrong columns.

## [0.1.0] - 2026-11-01

First playable version: a turn-based, post-apocalyptic roguelike drawn in
Unicode. Find the Visitor Beacon 📡 on depth 10.

### Added
- Six paired attributes, eight skills that improve with use, and a universal
  `2d10 + modifier` check with margin tiers.
- Margin & Momentum combat, with stances and a bonus for consecutive hits.
- Fourteen civilian guns and thirteen real loads, with damage derived from
  muzzle energy and barrel length.
- Melee tools, scrapcraft weapons and rare Visitor artifacts.
- Animals, mutants and cryptids that die into life crystals 💎, used to heal
  and to trade at Autovends.
- Harvesting, crafting, hunger and timed status effects.

[Unreleased]: https://github.com/ronstr8/roguemoji/compare/v0.9.0...HEAD
[0.9.0]: https://github.com/ronstr8/roguemoji/compare/v0.8.0...v0.9.0
[0.8.0]: https://github.com/ronstr8/roguemoji/compare/v0.7.0...v0.8.0
[0.7.0]: https://github.com/ronstr8/roguemoji/compare/v0.6.0...v0.7.0
[0.6.0]: https://github.com/ronstr8/roguemoji/compare/v0.5.0...v0.6.0
[0.5.0]: https://github.com/ronstr8/roguemoji/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/ronstr8/roguemoji/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/ronstr8/roguemoji/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/ronstr8/roguemoji/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/ronstr8/roguemoji/releases/tag/v0.1.0
