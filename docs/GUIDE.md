# Player's guide

Everything about playing roguemoji: the keys, and the rules behind them. The
in-game help (`?`) has the short version.

You need a terminal of at least 80×24 with a font that renders emoji as double-width
(most modern terminals do).

## Controls

| Key | Action |
|---|---|
| arrows · `hjkl yubn` · numpad | move; bump to attack, harvest 🌿🚗📦, or use an Autovend 🏧 |
| Space / `.` | wait |
| `s` | shoot: fires at the only target in range, or lists them nearest first (Enter shoots) |
| `f` | fire: Tab/←→ cycle targets (shows exact hit %), `f`/Enter shoot |
| `r` | reload |
| `g` / `,` / `5` | pick up |
| `a` / `A` | absorb one crystal / absorb until healed |
| `d` | drink from water underfoot or next to you |
| `z` | nap until rested (a monster in view, a hit or any key wakes you) |
| `S` | cycle stance |
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
5. Past its effective range a gun loses accuracy each tile, and it can't fire at all past
   its max range: the effective range plus one tile per 400 fps. A snub .38 reaches about
   5 tiles; a .308 rifle reaches 25.

The inventory screen shows a table for each gun with every load it accepts: bullet weight,
muzzle velocity, range, damage and penetration, with the loaded one marked ▸. Each item in your
pack lists what an Autovend would pay for it and its weight. Press `s` there to sort by name,
weight or value, ascending then descending. The Autovend's Sell tab and the Visitor Lockbox use
the same order, and `s` works there too.

**Melee weapons** come in three kinds:
- scavenged tools: knife, hammer, crowbar, wrench, bat, machete, shovel, fire axe
- scrapcraft: nail bat, pipe spear, scrap cleaver, spiked knuckles, shock baton
- rare Visitor artifacts 💠: Phase Edge, Graviton Maul, Hum Filament, Siphon Gauntlet

**Crystals** 💎 drop from everything that dies. Absorb them to heal (Willpower improves
the yield), or spend them at Autovend kiosks. Crystals you spend can't heal you. Autovends
also buy gear, ammo, food and components for half their price (Tab in the kiosk).

**Hunger and thirst** drain every turn, thirst about three times as fast. Hungry is
only a warning; Starving lowers Brawn, Reflexes and Fortitude and costs HP. Thirsty
lowers Brains and Wit, and Parched lowers them further and costs HP faster. Bottled
Water is safe. Dirty Water from pumps, and pools you drink from with `d`, quench as
well but take a Survival check to keep down; boil Dirty Water (`m`) to make it safe.

**Rest** drains every turn too, twice as fast while fighting and faster under a heavy
load. Tired lowers Reflexes; Exhausted lowers Reflexes and Wit. At empty you collapse
💤 and sleep where you stand until you've recovered some, unless something hits you.
Press `z` to nap first: it lasts until you're fully rested, a monster comes into view, or
you press a key. Sleep heals a little, but hunger and thirst keep draining.

**Hope** wears down slowly on its own and swings with **encounters**: things happening
on the map that you walk into. A 🤝 Friendly Stranger or a 🔥 Campfire Circle restores
it, and sometimes hands you something. Horrors like 😱 Witness Horrors erode it, unless
a Willpower check halves the blow. Places like the 🧘 Peaceful Sanctum stay put and
restore a little each time you bump them. You can tell them apart from across the map
by their glow: green for hopeful, pale yellow for a gamble, red for a horror. Downcast lowers Willpower; Despairing lowers
it further, and Wit too. Willpower powers crystal healing and resisting dread, so
despair makes 💎 heal less and terror land more often. It never costs HP.

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
spare guns and armor in one, then take them out of a lockbox in a later phase.
