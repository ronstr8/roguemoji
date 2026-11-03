# Game data

Content lives here as TOML, one file per thing. A file's **id** is its name
without `.toml` (`feral_dog.toml` → `feral_dog`).

## Where files are read from

1. **Built-in:** everything in this directory is compiled into the binary, so a
   release build runs on its own. Edit a file here and rebuild to change it.
2. **Drop-ins:** at startup the game also reads `data/<category>/*.toml` next to
   the executable, then from the directory in `ROGUEMOJI_DATA`, if set:

   ```sh
   ROGUEMOJI_DATA=~/my-mod roguemoji      # reads ~/my-mod/monsters/*.toml
   ```

   A drop-in with a new id adds content. One with an existing id replaces it
   (put a `rat.toml` in your mod to rebalance rats).

All files are checked at startup. A typo'd field, an unknown item or a bad value
stops the game with an error naming the file and line.

## Monsters — `monsters/*.toml`

```toml
name = "Feral Dog"
glyph = "🐕"                 # one emoji, drawn double-width
family = "animal"            # animal | mutant | cryptid
description = "Somebody's pet, three winters ago."

hp = 9
attack = 3                   # added to 2d10 to-hit
defense = 10                 # target number to hit it
damage = [2, 4]              # [min, max] per hit
armor = 0                    # optional, default 0; subtracted from damage
speed = 130                  # optional, default 100 (normal); 130 acts ~1.3× as often
crystals = [2, 4]            # 💎 dropped on death, [min, max]

# Optional extra effect on a solid hit. One of:
#   { effect = "poison", potency = 1, turns = 8 }   lose potency HP every other turn
#   { effect = "bleed",  potency = 1, turns = 3 }   lose potency HP every turn
#   { effect = "stun",   turns = 2 }
#   { effect = "dread" }                            Willpower check or be stunned
on_hit = { effect = "bleed", potency = 1, turns = 3 }

# Optional crafting components left behind, each rolled separately.
drops = [{ item = "raw_meat", chance = 0.6 }, { item = "bone", chance = 0.3 }]

[spawn]
min_depth = 1                # first depth it appears on
max_depth = 6                # optional; never spawns deeper than this
weight = 8                   # relative commonness (rat 10 … cryptids 2); 0 = never spawns
```

**Drop items** are component ids: any file in `components/`, including your own.

**How spawn weight plays out.** On each level, every eligible monster gets
`weight × 3`. That drops to a third once the level is more than 4 depths past
its `min_depth` (common critters thin out). Cryptids get `+ depth / 3` on top.
Monsters spawned deeper than their `min_depth` also get +1/6 of their HP per
extra depth.

## Components — `components/*.toml`

Crafting materials: harvested from nodes, dropped by monsters, found on the floor.

```toml
name = "Duct Tape"
glyph = "🩹"
description = "Holds the world together."
weight = 0.3                 # pounds
loot_weight = 1              # optional; relative chance among floor components.
                             # 0 or absent = never found lying around
```

Some components are named by the game's own tables: crafting recipes, harvest
nodes (🌿 🚗 🧰 …) and starting kits. You can override those files to change
names, glyphs, weights and loot chances. You can't remove them; the game
checks at startup that they all exist. A new component can be dropped by a
monster (`drops`) or found on the floor (`loot_weight`). Recipes can't use it
yet, because recipes are still defined in code.
