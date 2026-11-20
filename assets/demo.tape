# The README's demo GIF, recorded with VHS (https://github.com/charmbracelet/vhs).
# Re-record it with `make demo` after anything on screen changes.
#
# The world is seed 1312 and every move below is scripted, so each recording
# plays the same: pick the Ex-Deputy, name her, meet a Friendly Stranger, shoot
# a feral dog, a raccoon, a rat and another raccoon, grab the loot, then show
# the inventory's ballistics table and the character sheet. If a game change
# moves things around, replay the keys by hand with
# `make run SEED=1312` to fix them.

Output assets/demo.gif

Require ttyd

Set Shell "bash"
Set FontFamily "DejaVu Sans Mono"
Set FontSize 14
Set Width 1000
Set Height 620
Set Padding 12
Set TypingSpeed 90ms
Set Theme "Builtin Dark"

# Start out of sight, in a throwaway save folder so no real save is touched.
Hide
Type "./target/release/roguemoji --seed 1312 --save-dir $(mktemp -d)"
Enter
Sleep 1s
Show

# The title screen: browse the backgrounds, then take the Ex-Deputy.
Sleep 1.5s
Down
Sleep 1s
Down
Sleep 1s
Up
Sleep 1.2s
Enter
Sleep 700ms
Type "Vega"
Sleep 600ms
Enter
Sleep 2s

# Walk over to the stranger by the fire.
Type "b"
Sleep 300ms
Type "h"
Sleep 300ms
Type "h"
Sleep 3.5s
Enter
Sleep 800ms

# Six shots: `f` aims (showing the odds), the second `f` fires.
Type "f"
Sleep 1.4s
Type "f"
Sleep 900ms
Type "f"
Sleep 700ms
Type "f"
Sleep 1.2s
Type "f"
Sleep 1s
Type "f"
Sleep 900ms
Type "f"
Sleep 700ms
Type "f"
Sleep 1.2s
Type "f"
Sleep 700ms
Type "f"
Sleep 1s
Type "f"
Sleep 700ms
Type "f"
Sleep 1.5s

# Loot, then the inventory with the pistol's load table.
Type "b"
Sleep 400ms
Type "g"
Sleep 1s
Type "i"
Sleep 1.2s
Up
Sleep 300ms
Up
Sleep 300ms
Up
Sleep 3.5s
Escape
Sleep 600ms

# The character sheet.
Type "c"
Sleep 3.5s
Escape
Sleep 1.5s
