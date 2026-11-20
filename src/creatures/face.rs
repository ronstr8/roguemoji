//! The player's face on the map, which shows how they're holding up, loaded
//! from `data/faces/*.toml` (see `data/README.md`).
//!
//! Each face names one condition: low health, a need at or past a stage, or
//! sleep. Of the faces whose condition holds, the highest `rank` shows. The
//! one face with no condition is the default. A face can also flash a
//! background color, to catch the eye when things are going badly.

use serde::Deserialize;

use crate::color::{self, Rgb};
use crate::content::{self, Problems};
use crate::creatures::needs::Need;

/// A handle to a loaded face: an index into the face table.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Face(u16);

#[derive(Debug)]
pub struct FaceDef {
    /// The file name without `.toml`, e.g. `parched`.
    pub id: String,
    pub glyph: String,
    /// When several faces apply, the highest rank shows. Ranks are unique.
    pub rank: u32,
    pub when: When,
    pub flash: Option<Flash>,
}

/// A face's blinking background: lit for the first half of every period.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Flash {
    pub color: Rgb,
    /// One full blink, on and off, in milliseconds.
    pub every_ms: u32,
}

impl Flash {
    /// Whether the color is showing `elapsed_ms` after the clock started.
    pub fn lit(&self, elapsed_ms: u128) -> bool {
        let every = u128::from(self.every_ms);
        elapsed_ms % every < every / 2
    }
}

/// What has to be true for a face to show.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum When {
    /// The default face, when nothing else applies.
    Always,
    Asleep,
    /// HP below this percentage of max.
    HealthBelow(u32),
    /// A need at the stage at this index in its `stages`, or a lower one.
    Need {
        need: Need,
        stage: usize,
    },
}

impl Face {
    /// The face that shows for the conditions `holds` says are true.
    pub fn pick(holds: impl Fn(When) -> bool) -> Face {
        Face::all()
            .filter(|f| holds(f.def().when))
            .max_by_key(|f| f.def().rank)
            .expect("a default face (checked at load)")
    }
}

// ── File format ──

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawFace {
    glyph: String,
    rank: u32,
    #[serde(default)]
    health_below: Option<u32>,
    #[serde(default)]
    need: Option<String>,
    #[serde(default)]
    stage: Option<String>,
    #[serde(default)]
    asleep: bool,
    #[serde(default)]
    flash: Option<RawFlash>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawFlash {
    color: String,
    every_ms: u32,
}

/// Minimum blink period: faster than this is a strobe, not a warning.
const MIN_FLASH_MS: u32 = 200;

fn build_flash(raw: &RawFlash, p: &mut Problems) -> Option<Flash> {
    p.check(raw.every_ms >= MIN_FLASH_MS, format!("flash.every_ms must be at least {MIN_FLASH_MS}"));
    let color = color::parse(&raw.color);
    p.check(color.is_some(), format!("flash.color `{}` must be #RRGGBB or an X11 color name", raw.color));
    color.map(|color| Flash { color, every_ms: raw.every_ms })
}

fn build(id: String, raw: RawFace) -> Result<FaceDef, Vec<String>> {
    let mut p = Problems::default();
    p.check(!raw.glyph.trim().is_empty(), "glyph is empty");
    let triggers = usize::from(raw.health_below.is_some()) + usize::from(raw.need.is_some()) + usize::from(raw.asleep);
    p.check(triggers <= 1, "give at most one of health_below, need or asleep");
    p.check(raw.health_below.is_none_or(|n| (1..=100).contains(&n)), "health_below must be 1-100 (a percentage)");
    p.check(raw.need.is_some() == raw.stage.is_some(), "need and stage go together");
    let when = when(&raw, &mut p);
    let flash = raw.flash.as_ref().and_then(|f| build_flash(f, &mut p));
    p.finish(|| FaceDef { id, glyph: raw.glyph, rank: raw.rank, when, flash })
}

fn when(raw: &RawFace, p: &mut Problems) -> When {
    if let Some(n) = raw.health_below {
        return When::HealthBelow(n);
    }
    if raw.asleep {
        return When::Asleep;
    }
    let (Some(need_id), Some(stage_name)) = (&raw.need, &raw.stage) else { return When::Always };
    let Some(need) = Need::lookup(need_id) else {
        p.check(false, format!("unknown need `{need_id}`"));
        return When::Always;
    };
    match need.def().stages.iter().position(|s| s.name.eq_ignore_ascii_case(stage_name)) {
        Some(stage) => When::Need { need, stage },
        None => {
            p.check(false, format!("need `{need_id}` has no stage `{stage_name}`"));
            When::Always
        }
    }
}

content::registry!(Face, FaceDef, "face", load);

fn load() -> Result<Vec<FaceDef>, String> {
    let defs = content::load_defs("faces", build)?;
    let defaults: Vec<&str> = defs.iter().filter(|d| d.when == When::Always).map(|d| d.id.as_str()).collect();
    if defaults.len() != 1 {
        return Err(format!("data/faces needs exactly one face with no condition (the default); found {defaults:?}"));
    }
    for (i, a) in defs.iter().enumerate() {
        if let Some(b) = defs[i + 1..].iter().find(|b| b.rank == a.rank) {
            return Err(format!("data/faces: {} and {} share rank {}; ranks must differ", a.id, b.id, a.rank));
        }
    }
    Ok(defs)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(text: &str) -> Result<FaceDef, String> {
        let raw: RawFace = toml::from_str(text).map_err(|e| e.to_string())?;
        build("t".into(), raw).map_err(|e| e.join("; "))
    }

    #[test]
    fn reads_each_condition() {
        assert_eq!(parse("glyph = \"🙂\"\nrank = 0").unwrap().when, When::Always);
        assert_eq!(parse("glyph = \"💤\"\nrank = 1\nasleep = true").unwrap().when, When::Asleep);
        assert_eq!(parse("glyph = \"🤕\"\nrank = 2\nhealth_below = 25").unwrap().when, When::HealthBelow(25));
        let thirst = Need::named(crate::creatures::needs::THIRST);
        let parched = parse("glyph = \"🥵\"\nrank = 3\nneed = \"thirst\"\nstage = \"parched\"").unwrap();
        assert_eq!(parched.when, When::Need { need: thirst, stage: 2 });
    }

    #[test]
    fn reads_a_flash() {
        let f = parse("glyph = \"x\"\nrank = 1\nflash = { color = \"#C03a00\", every_ms = 800 }").unwrap();
        assert_eq!(f.flash, Some(Flash { color: (0xc0, 0x3a, 0x00), every_ms: 800 }));
        let named = parse("glyph = \"x\"\nrank = 1\nflash = { color = \"Fire Brick\", every_ms = 800 }").unwrap();
        assert_eq!(named.flash.map(|f| f.color), Some((178, 34, 34)));
        assert!(parse("glyph = \"x\"\nrank = 1\nflash = { color = \"blood\", every_ms = 800 }").is_err());
        assert!(parse("glyph = \"x\"\nrank = 1\nflash = { color = \"#c03a0g\", every_ms = 800 }").is_err());
        assert!(parse("glyph = \"x\"\nrank = 1\nflash = { color = \"#c03a00\", every_ms = 50 }").is_err());
    }

    #[test]
    fn a_flash_is_lit_for_the_first_half_of_each_blink() {
        let f = Flash { color: (255, 0, 0), every_ms: 1000 };
        assert!(f.lit(0) && f.lit(499) && f.lit(1200));
        assert!(!f.lit(500) && !f.lit(999));
    }

    #[test]
    fn rejects_bad_faces() {
        assert!(parse("glyph = \"x\"\nrank = 1\nasleep = true\nhealth_below = 5").unwrap_err().contains("at most one"));
        assert!(parse("glyph = \"x\"\nrank = 1\nhealth_below = 0").unwrap_err().contains("1-100"));
        assert!(parse("glyph = \"x\"\nrank = 1\nneed = \"thirst\"").unwrap_err().contains("go together"));
        assert!(
            parse("glyph = \"x\"\nrank = 1\nneed = \"thirst\"\nstage = \"Damp\"").unwrap_err().contains("no stage")
        );
        assert!(
            parse("glyph = \"x\"\nrank = 1\nneed = \"mirth\"\nstage = \"Glum\"").unwrap_err().contains("unknown need")
        );
    }

    fn player() -> crate::creatures::Player {
        crate::creatures::Background::ALL[0].create("Tester".into())
    }

    #[test]
    fn a_healthy_player_smiles() {
        assert_eq!(player().face().glyph, "🙂");
    }

    #[test]
    fn a_need_shows_at_its_stage_and_past_it() {
        let mut p = player();
        p.thirst.value = 200;
        assert_eq!(p.face().glyph, "😓", "Thirsty");
        p.thirst.value = 0;
        assert_eq!(p.face().glyph, "🥵", "Parched outranks Thirsty");
    }

    #[test]
    fn the_worst_condition_wins() {
        let mut p = player();
        p.thirst.value = 0;
        p.hp = p.max_hp() * 4 / 10;
        assert_eq!(p.face().glyph, "🥵", "parched outranks half health");
        p.hp = 1;
        assert_eq!(p.face().glyph, "🤕", "near death outranks any need");
        p.sleep = Some(crate::creatures::player::Sleep::Nap);
        assert_eq!(p.face().glyph, "💤", "sleep shows over everything");
    }

    #[test]
    fn stock_faces_load() {
        assert!(load().is_ok(), "{:?}", load().err());
    }
}
