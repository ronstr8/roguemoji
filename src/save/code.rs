//! Resume codes: how a player finds their save again on a shared server,
//! where every player runs as the same system user.
//!
//! A code is three words and four digits, like `ember-otter-mesa-4127`:
//! about 37 random bits, so too many to guess, but short enough to write down.

use crate::dice::Dice;

const WORDS_TXT: &str = include_str!("words.txt");
const WORDS_PER_CODE: usize = 3;

fn words() -> impl Iterator<Item = &'static str> {
    WORDS_TXT.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#'))
}

/// A fresh random code.
pub fn generate() -> String {
    let mut dice = Dice::from_entropy();
    let list: Vec<&str> = words().collect();
    let mut parts: Vec<String> = (0..WORDS_PER_CODE).map(|_| dice.pick(&list).to_string()).collect();
    parts.push(format!("{:04}", dice.range(0, 9999)));
    parts.join("-")
}

/// The code a player typed, tidied into its canonical form, or `None` if it
/// can't be one. Case, spaces and underscores don't matter. Only a valid
/// code comes back, so the result is always safe to use as a file name.
pub fn normalize(typed: &str) -> Option<String> {
    let lower = typed.trim().to_lowercase().replace(['_', ' '], "-");
    let parts: Vec<&str> = lower.split('-').filter(|p| !p.is_empty()).collect();
    let (digits, named) = parts.split_last()?;
    let words_ok = named.len() == WORDS_PER_CODE && named.iter().all(|p| words().any(|w| w == *p));
    let digits_ok = digits.len() == 4 && digits.bytes().all(|b| b.is_ascii_digit());
    (words_ok && digits_ok).then(|| parts.join("-"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_word_list_has_256_distinct_plain_words() {
        let list: Vec<&str> = words().collect();
        let mut sorted = list.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!((list.len(), sorted.len()), (256, 256));
        assert!(list.iter().all(|w| w.bytes().all(|b| b.is_ascii_lowercase())));
    }

    #[test]
    fn generated_codes_read_back() {
        for _ in 0..20 {
            let code = generate();
            assert_eq!(normalize(&code), Some(code.clone()));
        }
    }

    #[test]
    fn typed_codes_are_forgiving_about_case_and_spacing() {
        assert_eq!(normalize("  Ember Otter_mesa--0412 ").as_deref(), Some("ember-otter-mesa-0412"));
    }

    #[test]
    fn rejects_anything_else() {
        for bad in ["", "ember-otter-mesa", "ember-otter-mesa-412", "ember-otter-blorp-4127", "../../etc/passwd"] {
            assert_eq!(normalize(bad), None, "{bad}");
        }
        assert_eq!(normalize("ember-otter-mesa-pine-4127"), None, "four words");
    }
}
