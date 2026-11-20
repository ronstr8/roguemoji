# Developer notes

How roguemoji is built, released and hosted, and where things live in the
code. For the game itself, see the [player's guide](GUIDE.md). For content
files, see [data/README.md](../data/README.md).

## Commands

```
cargo run --release            # play
cargo run --release -- --seed 42   # reproducible world
cargo run --release -- --version   # print the version
cargo test                     # unit + soak tests
make static                    # self-contained Linux binary (musl), no shared-lib deps
make dist                      # package that binary with docs and data/ into dist/
make webplay                   # host public play over SSH and in the browser (deploy/)
```

`make` alone lists every target. `make ci` (fmt check, clippy with
`-D warnings`, release-mode tests) must pass before each commit.

## Hosting

To let anyone play on your server, over `ssh roguemoji@your.host` or in a
browser, see [deploy/README.md](../deploy/README.md). It has a locked-down sshd
config, a sandboxed ttyd service, nginx and Apache snippets, and a Makefile that
installs them: `make webplay`.

## Versions

The version lives in `Cargo.toml` and shows on the title screen. Each release is
tagged `vX.Y.Z`, and [CHANGELOG.md](../CHANGELOG.md) lists what changed. Add new
entries under **Unreleased** as you go.

To cut a release:

1. In CHANGELOG.md, rename **Unreleased** to `[X.Y.Z] - date` and add its compare link.
2. Set the same version in Cargo.toml, commit, `git tag -a vX.Y.Z`, and push the
   commit and the tag.
3. Run `make publish`. It checks that the tree is clean, HEAD carries the tag,
   the tag is on GitHub and the changelog has the section. Then it runs
   `make ci`, builds the tarball with `make dist`, and creates the GitHub
   release with the changelog section as its notes.

Release tarballs include the stock `data/` folder next to the binary. The game
reads it at startup, so players can edit those files and see the change
without building anything.

## Project layout

```
data/
  monsters/        one TOML file per monster (see data/README.md)
  components/      one TOML file per crafting component
  consumables/     food, drink and medicine
  needs/           hunger, thirst, rest and hope: drain rates, stages and sleep
  faces/           the player's face on the map, by health and needs
  encounters/      scenes and places on the map, and what they do
  recipes/         crafting recipes
  melee/ armor/ packs/   wieldable and wearable gear
  cartridges/ ammo/ guns/  firearms and their ballistics
build.rs           embeds data/ into the binary
assets/            README images (logo, mascot)
docs/              player's guide and these developer notes
deploy/            configs for hosting public play over SSH and in the browser
src/
  main.rs          entry point, content validation, terminal setup
  content.rs       finds and parses data files (built-in + drop-ins)
  app.rs           screen state machine and input dispatch
  input.rs         key → command mapping
  dice.rs          RNG with dice helpers
  geometry.rs      Pos, Dir, lines, rects
  effects.rs       timed status effects (poison, bleed, buffs…)
  combat.rs        Margin & Momentum resolution, stances
  crafting.rs      recipe loader and crafting outcomes
  stats/           attributes, skills, the 2d10 check
  items/           ballistics, guns, melee, armor, components, consumables, inventory, loot tables
  creatures/       bestiary loader (species), monster, player, backgrounds
  world/           tiles, map, features, harvest nodes, encounters, FOV, level generation
  game/            Game state; level, turn, ai, actions each add an `impl Game`
  ui/              canvas (wide-char aware), terminal (diffed output), HUD, map view, pop-up screens
```

All game content is data-driven: monsters, components, consumables, recipes,
weapons, armor, packs, guns, cartridges and ammo are TOML files under `data/`,
and you can add or override them without recompiling (see
[data/README.md](../data/README.md)). Rules stay in code: load types, gun actions,
melee specials and effect kinds are enums that content files refer to by name. Game rules never touch the terminal, and views never mutate the game.
