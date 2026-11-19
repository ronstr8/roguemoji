# roguemoji

Emoji roguelike in Rust (crossterm). Imperial units throughout: pounds,
grains, fps, inches. Distances on the map are tiles.

## Commands

- `make ci`: fmt-check, clippy with `-D warnings`, and release-mode tests.
  It must pass before every commit.
- `make run SEED=42`: play a reproducible world.
- `make windows`: cross-compile the Windows exe (MinGW).
- `make dist` / `make publish`: package and release. See "Versions" in `docs/DEVELOPER.md`.

## Content is data

- Monsters, items, recipes, ammo and their numbers live in `data/<category>/*.toml`.
  Don't add content or tuning numbers to Rust. Code holds only what carries
  rules (load types, effect kinds, burden levels).
- `build.rs` embeds `data/` in the binary. At startup the game also reads
  drop-ins from `data/` beside the exe and from `$ROGUEMOJI_DATA`.
- Every file is validated at startup (`content::Problems`). A new field gets a
  check that names the file and what's wrong, and `deny_unknown_fields` stays on.
- A new or changed data field is documented in `data/README.md` in the same commit.
- Derive numbers from real-world figures where they exist (see the ballistics
  model in `src/items/ballistics.rs`) instead of hand-tuning them.

## Code layout

- Game rules go on model types (`Burden::lighter`, `Player::carry_capacity`).
  `src/ui/` only formats and draws. It never decides outcomes.
- Shared pop-up pieces (`popup`, `footer`, `paragraph`, `draw_load`) live in
  `src/ui/screens/mod.rs`. Reuse them, and add to them rather than copying
  between screens.
- Weights are `u32` hundredths of a pound. Format them with `weight::fmt_lb`.
- Every module has a `//!` doc comment, and public items get `///` comments
  that say what the item means.

## Tests

- Rule changes get a unit test, in the module or in `src/game/tests.rs`.
- Screen changes get a test render (see the test in `src/ui/screens/shop.rs`).
- Use `Game::new(.., Some(seed))` for determinism.
