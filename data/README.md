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
price = 2                    # optional, default 2; Autovend price of one, in 💎. 0 = never sold
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
price = 5                    # optional, default 5; Autovend price in 💎. 0 = never sold

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

Guns, melee weapons, armor and packs share one optional `loot` field that says how
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

## Autovend prices

Anything can carry a `price` in 💎, written as a plain number or scaled by depth:

```toml
price = 30                              # same everywhere
price = { base = 25, per_depth = 3 }    # 25 + 3 × depth (28 on depth 1, 40 on depth 5)
```

An item with no price (or 0) is never sold, even if it turns up as loot. The
defaults are 5 for consumables and 2 for components; guns, ammo, melee weapons,
armor and packs have no default, so a new one is only sold if you give it a price.

Each Autovend stocks three boxes of ammo, two foods, a pair of components and a
gun. It may also have armor (50%), a pack (35%) and, from depth 4, a Visitor
artifact (30%). Each slot is picked from the priced items of its kind by `loot`
(by `loot_weight` for food and components). Guns, ammo, armor and packs are
rolled as if one level deeper. A slot with nothing priced to sell is left empty.
Ammo `price` is for a full box; a partial box costs its share, rounded up.

Stock prices: guns by quality, from 15 (snub .38) to 55 (AR-15), +3 per depth; armor 25, +3 per depth;
packs 18, +3 per depth; Visitor artifacts 60, +5 per depth. Tools and crafted
weapons have no price.

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
price = { base = 60, per_depth = 5 }   # optional; see "Autovend prices"
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
price = { base = 25, per_depth = 3 }   # optional; see "Autovend prices"
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
price = { base = 18, per_depth = 3 }   # optional; see "Autovend prices"
```

## Guns and ammunition

Firearms are built from three kinds of file: a **cartridge** (the round's
size), **ammo** (a specific load of that cartridge with real ballistic figures),
and a **gun** (chambered for one cartridge). Damage isn't set by hand. It comes
from the bullet's muzzle energy, which depends on the gun's barrel length:

- velocity = `ref_fps + (barrel_in − ref_barrel_in) × fps_per_inch`, kept within 60–125% of `ref_fps`
- energy = ½ × mass × velocity² (grains and fps converted to SI)
- damage ≈ √joules ÷ 3 × the load type's multiplier, split across `projectiles`
- penetration = +1 at 1400 fps or faster, +2 at 2000 fps, plus the load type's modifier

### Cartridges — `cartridges/*.toml`

```toml
name = ".357 Magnum"
round_weight = 0.03          # pounds per round
also_fires = ["38_special"]  # optional; other cartridges this chamber can safely fire
```

### Ammo — `ammo/*.toml`

```toml
label = "9mm 124gr JHP"      # shown in the inventory and HUD
cartridge = "9mm"
load = "hollow_point"        # see the table below
box_size = 10                # rounds in a box when found (you find half to all of it)
price = 7                    # optional; Autovend price of a full box (see "Autovend prices")

grains = 124.0               # projectile mass; for shot, all pellets together
projectiles = 1              # optional, default 1; pellets per shell for buckshot
ref_fps = 1150.0             # published muzzle velocity…
ref_barrel_in = 4.0          # …from a barrel this long
fps_per_inch = 18.0          # velocity gained or lost per inch of barrel
```

| load | damage | pen | notes |
|---|---|---|---|
| `lead_round_nose` | ×0.95 | +0 | |
| `fmj` | ×1.0 | +1 | |
| `hollow_point` | ×1.25 | −1 | |
| `soft_point` | ×1.2 | +0 | |
| `buckshot` | ×1.0 | −1 | +2 to hit within 4 tiles; range capped at 5; pellets spread with distance |
| `slug` | ×1.1 | +1 | from a smoothbore: −1 to hit, but range at least 8 |

### Guns — `guns/*.toml`

```toml
name = "9mm Service Pistol"
description = "Full-size polymer duty pistol. Reliable and high capacity."
weight = 1.9                 # pounds, unloaded (loaded rounds add their own weight)

cartridge = "9mm"
action = "semi_auto"         # semi_auto | revolver | pump | lever | bolt | break_open
skill = "small_arms"         # small_arms | long_arms
barrel_in = 4.5
capacity = 15
accuracy = 1                 # optional, default 0; to-hit modifier
effective_range = 6          # tiles before range penalties start
rifled = true                # false for shotguns
loot = { weight = 8, per_depth = 1 }   # see "Loot chances for gear"
price = { base = 28, per_depth = 3 }   # optional; see "Autovend prices"
```

**Reload time** comes from `action`: `semi_auto` and `break_open` take 1 turn,
`bolt` takes 2, and `revolver`, `pump` and `lever` take 3.

**Range.** Each tile past `effective_range` costs −2 to hit with `small_arms`
and −1 with `long_arms`. A `long_arms` gun is also −2 to hit at point-blank.

Every gun's cartridge needs at least one ammo file that fits it (directly or
through `also_fires`), or the game won't start. Starting kits name the guns
`target_pistol_22`, `service_pistol_9mm`, `snub_revolver_38` and
`single_shot_20ga`, and the ammo `22lr`, `9mm_jhp`, `9mm_fmj`, `38_special`
and `20ga_buck`.
