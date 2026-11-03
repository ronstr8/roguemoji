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
#   { kind = "poison", potency = 1, turns = 8 }   lose potency HP every other turn
#   { kind = "bleed",  potency = 1, turns = 3 }   lose potency HP every turn
#   { kind = "stun",   turns = 2 }
#   { kind = "dread" }                            Willpower check or be stunned
on_hit = { kind = "bleed", potency = 1, turns = 3 }

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

Some components are named by the game's own tables: harvest nodes (🌿 🚗 🧰 …)
and starting kits. You can override those files to change names, glyphs,
weights and loot chances. You can't remove them; the game checks at startup
that they all exist. A new component can be dropped by a monster (`drops`),
found on the floor (`loot_weight`), and used in recipes.

## Consumables — `consumables/*.toml`

Food, drink and medicine. Press Enter on one in the inventory to use it.

```toml
name = "Glowcap Brew"
glyph = "🧉"
description = "Shimmering mushroom tea. Thoughts come fast and bright."
weight = 1                   # pounds

satiety = 40                 # hunger restored
heal = 0                     # optional, default 0; HP restored at once
cures = false                # optional, default false; true removes poison and bleeding
loot_weight = 0              # optional; relative chance among food found on the floor
                             # and sold at Autovends. 0 or absent = crafted only
price = 5                    # optional, default 5; Autovend price in 💎

# Optional timed effects. Each is one of:
#   { kind = "boost", attribute = "brawn", potency = 2, turns = 120 }
#        attribute: brawn | brains | reflexes | wit | fortitude | willpower
#        (a negative potency lowers it)
#   { kind = "regen",     potency = 1, turns = 20 }   regain potency HP every other turn
#   { kind = "fortified", potency = 1, turns = 150 }  + potency armor
#   { kind = "focus",     potency = 2, turns = 80 }   + potency to every to-hit roll
#   { kind = "poison" | "bleed", potency = 1, turns = 5 }
#   { kind = "stun", turns = 2 }
effects = [
    { kind = "boost", attribute = "brains", potency = 2, turns = 120 },
    { kind = "boost", attribute = "wit", potency = 2, turns = 120 },
]
```

Starting kits name `canned_peaches`, `bottled_water` and `hardtack`, so those
must exist (you can still override them).

## Recipes — `recipes/*.toml`

What the crafting menu (`m`) can make. Every craft is a Crafting check against
`difficulty`. A critical success makes a double batch of food, or refunds the
first ingredient for gear.

```toml
makes = { food = "glowcap_brew" }   # or { melee = "nail_bat" }, { armor = "scrap_vest" },
                                    #    { pack = "rag_satchel" }
difficulty = 11                     # target for 2d10 + Crafting (8 easy … 15 hard)
ingredients = { glowcap = 1, dirty_water = 1 }   # component id = count, in display order
uses_up = "kitchen_knife"           # optional; a melee weapon consumed as well
```

Each output names an id from the matching folder, including your own files:
`food` → `consumables/`, `melee` → `melee/`, `armor` → `armor/`, `pack` → `packs/`.
`uses_up` names a file in `melee/`.

The menu groups food, then weapons, armor and packs, each sorted by difficulty
and then by name. A file id doesn't have to match what it makes, so you can add
a second recipe for the same item (e.g. `grilled_meat_cactus.toml`).

## Loot chances for gear

Melee weapons, armor and packs share one optional `loot` field that says how
often each turns up on the floor and in Autovends, by depth:

```toml
loot = { weight = 2, per_depth = 1, min_depth = 1 }
# chance weight at depth d = weight + per_depth × d, once d >= min_depth.
# All three are optional (defaults 0, 0, 1). No `loot` = never found; craft it instead.
```

Weights are relative to the other items of the same kind. The stock values:

| | weight | per_depth | min_depth | at depth 1 / 5 / 8 |
|---|---|---|---|---|
| Leather Jacket | 10 | | | 10 / 10 / 10 |
| Kevlar Vest | 2 | 1 | | 3 / 7 / 10 |
| Riot Gear | | 1 | 4 | 0 / 5 / 8 |
| Visitor Weave | 1 | | 5 | 0 / 1 / 1 |

Autovends roll their armor and packs as if one level deeper.

## Melee weapons — `melee/*.toml`

```toml
name = "Fire Axe"
glyph = "🪓"
description = "Pick-head axe from a station wall."
weight = 6                   # pounds

origin = "tool"              # tool | crafted | alien
skill = "melee"              # melee | brawling
damage = [3, 8]              # [min, max]; your Brawn bonus is added
accuracy = -2                # optional, default 0; added to the to-hit roll
penetration = 2              # optional, default 0; ignores this much armor
special = "bleed"            # optional: bleed | stun | knockback | ignore_armor | drain
loot = { weight = 1 }        # see "Loot chances for gear"
```

**Specials** trigger on solid hits. `bleed` makes the target bleed. `stun` can
stun it. `knockback` shoves it a tile back and stuns it briefly. `ignore_armor`
skips armor entirely. `drain` heals you for a share of the damage.

**Where weapons drop.** A floor item is first picked by kind. "Tool" floor
items choose among every non-alien weapon by `loot`. "Visitor artifact" items
(from depth 3) and the rare Autovend artifact (from depth 4) choose among
`origin = "alien"` weapons. Stock crafted weapons have no `loot`, so you only
get them from recipes.

Starting kits name `crowbar`, `pipe_wrench`, `baseball_bat` and `kitchen_knife`.

## Armor — `armor/*.toml`

```toml
name = "Kevlar Vest"
glyph = "🦺"
description = "Soft body armor. Stops teeth almost as well as bullets."
weight = 8                   # pounds

armor = 2                    # subtracted from incoming damage (less the attack's penetration)
dodge_penalty = 0            # optional, default 0; subtracted from your Defense
loot = { weight = 2, per_depth = 1 }
```

Starting kits name `kevlar_vest` and `leather_jacket`.

## Packs — `packs/*.toml`

```toml
name = "Hiking Pack"
glyph = "🎒"
description = "Hip belt, sternum strap, a dozen pockets."
weight = 3                   # pounds

capacity = 25                # extra pounds you can carry before you're Burdened
loot = { weight = 3, per_depth = 1 }
```
