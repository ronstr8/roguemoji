use super::*;
use crate::creatures::Background;

fn game(seed: u64) -> Game {
    Game::new(Background::ALL[0].create("Tester".into()), Some(seed))
}

fn json(g: &Game) -> String {
    serde_json::to_string(g).unwrap()
}

/// A store in its own empty temp folder, removed when the test ends.
struct TempStore(Store);

impl TempStore {
    fn new(name: &str, shared: bool) -> Self {
        let dir = std::env::temp_dir().join(format!("roguemoji-test-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        TempStore(Store::new(dir, shared))
    }
}

impl Drop for TempStore {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0.dir);
    }
}

#[test]
fn a_run_reads_back_exactly() {
    let s = TempStore::new("exact", false);
    let mut g = game(7);
    for _ in 0..30 {
        g.wait();
    }
    s.0.write(LOCAL_SLOT, &g).unwrap();
    assert_eq!(json(&s.0.read(LOCAL_SLOT).unwrap()), json(&g));
}

#[test]
fn a_resumed_run_rolls_the_same_dice() {
    let s = TempStore::new("dice", false);
    let mut original = game(11);
    s.0.write(LOCAL_SLOT, &original).unwrap();
    let mut resumed = s.0.read(LOCAL_SLOT).unwrap();
    for _ in 0..200 {
        original.wait();
        resumed.wait();
    }
    assert_eq!(json(&resumed), json(&original));
}

#[test]
fn taking_a_run_removes_its_file() {
    let s = TempStore::new("take", false);
    s.0.write(LOCAL_SLOT, &game(3)).unwrap();
    assert_eq!(s.0.saved_local().as_deref(), Some(LOCAL_SLOT));
    assert!(s.0.take(LOCAL_SLOT).is_ok());
    assert_eq!(s.0.saved_local(), None);
    assert!(s.0.take(LOCAL_SLOT).err().unwrap().contains("No saved run"));
}

#[test]
fn shared_runs_are_kept_apart_by_code() {
    let s = TempStore::new("shared", true);
    let (a, b) = (s.0.new_slot(), s.0.new_slot());
    assert_ne!(a, b);
    assert!(code::normalize(&a).is_some());
    s.0.write(&a, &game(1)).unwrap();
    s.0.write(&b, &game(2)).unwrap();
    assert_eq!(json(&s.0.take(&b).unwrap()), json(&game(2)));
    assert_eq!(json(&s.0.take(&a).unwrap()), json(&game(1)));
    assert_eq!(s.0.saved_local(), None, "a shared store has no single slot");
}

#[test]
fn other_versions_and_junk_get_a_clear_message() {
    let old = format!(r#"{{"format":0,"version":"0.1.0","game":{}}}"#, json(&game(5)));
    assert!(parse(&old).err().unwrap().contains("saved by roguemoji 0.1.0"));
    assert!(parse("not json").err().unwrap().contains("isn't a roguemoji save"));
}

#[test]
fn content_missing_from_this_version_is_named() {
    let s = TempStore::new("missing", false);
    s.0.write(LOCAL_SLOT, &game(9)).unwrap();
    let path = s.0.path(LOCAL_SLOT);
    let text = fs::read_to_string(&path).unwrap();
    let melee = game(9).player.melee.expect("the first background starts armed").def().id.clone();
    fs::write(&path, text.replace(&format!("\"{melee}\""), "\"laser_sword\"")).unwrap();
    let err = s.0.read(LOCAL_SLOT).err().unwrap();
    assert!(err.contains("no melee weapon `laser_sword`"), "{err}");
}
