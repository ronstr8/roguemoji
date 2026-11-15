# Changelog

All notable changes to roguemoji are listed here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow
[Semantic Versioning](https://semver.org/). While the game is below 1.0, a minor
version can change rules, controls or data-file formats.

## [Unreleased]

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

[Unreleased]: https://github.com/ronstr8/roguemoji/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/ronstr8/roguemoji/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/ronstr8/roguemoji/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/ronstr8/roguemoji/releases/tag/v0.1.0
