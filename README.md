# roguemoji 💥

A post-apocalyptic, near-future roguelike drawn entirely in Unicode, written in Rust.

The Visitors came, took nothing, and left. Since then, animals have mutated, cryptids have
stopped hiding, and everything that dies crystallizes into 💎 life crystals. Somewhere ten
levels down, their Beacon 📡 still hums. Go find it.

```
cargo run --release            # play
cargo run --release -- --seed 42   # reproducible world
cargo test                     # unit + soak tests
make static                    # self-contained Linux binary (musl), no shared-lib deps
```

You need a terminal of at least 80×24 with a font that renders emoji as double-width
(most modern terminals do).

## Controls

| Key | Action |
|---|---|
| arrows · `hjkl yubn` · numpad | move; bump to attack, harvest 🌿🚗📦, or use an Autovend 🏧 |
| `.` / `5` | wait |
| `f` | fire: Tab/←→ cycle targets (shows exact hit %), `f`/Enter shoot |
| `r` | reload |
| `g` / `,` | pick up |
| `a` / `A` | absorb one crystal / absorb until healed |
| `s` | cycle stance |
| `>` | descend on 🔽 |
| `i` `c` `m` `?` | inventory · character sheet · crafting · help |
| `Q` | quit |

## Rules

**Attributes** come in physical/mental pairs: Brawn/Brains, Reflexes/Wit,
Fortitude/Willpower. A score of 5 is average; the bonus is `score − 5`.

**Skills** are Melee, Brawling, Small Arms, Long Arms, Dodge, Crafting, Scavenging and
Survival. Each averages the bonuses of two attributes (Melee uses Brawn + Reflexes, for
example). Skill modifier = `rank × 2 + attribute bonus`. When unskilled (rank 0), only
**half** of a positive attribute bonus applies. Penalties always apply in full. Skills rank
up through successful use.

**Checks** are `2d10 + modifier` against a target number. The margin sets the tier:
fumble (natural 2, or missing by 8+), fail, partial (0–3), success (4–8), critical (9+ or
natural 20). Every system uses the same tiers.

**Combat: Margin & Momentum.** An attack is a check against the defender's Defense.

| Margin | Result |
|---|---|
| < 0 | miss |
| 0–3 | graze: half damage after armor |
| 4–8 | hit: full damage after armor; weapon specials trigger |
| 9+ | crit: ×1.5 damage, ignores armor |

- **Stance:** Aggressive (+2 hit, −2 def, +1 melee damage), Balanced, or Guarded (−2 hit, +3 def).
- **Momentum:** each consecutive solid hit on the same target adds +1 to hit, up to +3. A
  miss, a switch of target, or taking damage resets it.

**Guns** are common civilian firearms: .22 pistols and rifles, a .38 snub, 9mm compact and
service pistols, a 1911, a .357 revolver (which also chambers .38 Special), a 9mm carbine,
20 and 12 gauge shotguns, a .30-30 lever gun, an SKS, an AR-15, and a .308 bolt rifle.
Damage is derived from physics rather than tuned by hand:

1. Each load has a real projectile weight and a published velocity at a reference barrel length.
2. The gun's barrel length shifts that velocity. Rimfire and pistol rounds gain little;
   magnums and rifle rounds gain a lot.
3. Muzzle energy is `½mv²` in joules, and damage is `√joules / 3`. This keeps the
   real-world order (.22 < .38 < 9mm < .357 < 5.56 < .308) while squeezing a 25× energy
   spread into about 5×.
4. Hollow and soft points wound more. FMJ, slugs and high-velocity rifle rounds penetrate
   armor. Buckshot hits more easily up close, but its pellets spread with distance.
   Smoothbores are less accurate with slugs, and long guns take a penalty at point-blank
   range.

The inventory screen shows each gun's fps, joules, damage and penetration for every load
it accepts.

**Melee weapons** come in three kinds:
- scavenged tools: knife, hammer, crowbar, wrench, bat, machete, shovel, fire axe
- scrapcraft: nail bat, pipe spear, scrap cleaver, spiked knuckles, shock baton
- rare Visitor artifacts 💠: Phase Edge, Graviton Maul, Hum Filament, Siphon Gauntlet

**Crystals** 💎 drop from everything that dies. Absorb them to heal (Willpower improves
the yield), or spend them at Autovend kiosks. Crystals you spend can't heal you.

**Harvesting and crafting.** Bump plants, pumps, wrecks, toolboxes, electronics and crates
to gather components with Survival or Scavenging. Then craft food and drink with timed
buffs, such as Glowcap Brew (+Brains/Wit), Grilled Meat (+Brawn), Mutant Stew
(+Fortitude, armor) and Cryptid Tonic. You can also craft scrap weapons and armor. The
crafting screen shows your exact success chance.

**Encumbrance.** Everything has a weight, including your worn and wielded gear
and the rounds in your gun. You can carry 45 lb, plus 6 lb for each point of
Brawn over 5 (or minus 6 lb per point under), plus whatever a worn pack 🎒
adds. Brawn buffs from food count. Above your capacity you're **Burdened**:
steps take 1.5 turns and you get −1 Defense. Above 1.5× you're **Strained**:
steps take 2 turns and you get −3 Defense. Above 2× you're **Overloaded** and
can't move until you drop something. Packs range from a craftable Rag Satchel
(+8 lb) to the Visitors' Grav Harness (+50 lb). The pack still has 26 slots;
stackable items share one slot.

**Visitor lockboxes** 🛅 all open onto one shared stash of 20 slots. Leave
spare guns and armor in one, then take them out of a lockbox on a deeper level.

## Project layout

```
src/
  main.rs          entry point, terminal setup
  app.rs           screen state machine and input dispatch
  input.rs         key → command mapping
  dice.rs          RNG with dice helpers
  geometry.rs      Pos, Dir, lines, rects
  effects.rs       timed status effects (poison, bleed, buffs…)
  combat.rs        Margin & Momentum resolution, stances
  crafting.rs      recipes
  stats/           attributes, skills, the 2d10 check
  items/           ballistics, guns, melee, armor, components, consumables, inventory, loot tables
  creatures/       bestiary (species), monster, player, backgrounds
  world/           tiles, map, features/harvestables, FOV, level generation
  game/            Game state; level, turn, ai, actions each add an `impl Game`
  ui/              canvas (wide-char aware), terminal (diffed output), HUD, map view, pop-up screens
```

Data is static and typed: every item, monster and recipe is an enum variant with a
`def()` table. Game rules never touch the terminal, and views never mutate the game.
