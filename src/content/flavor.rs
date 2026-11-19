//! Flavor text: a small markup that varies a line each time it's rolled.
//!
//! - `{ red | blue | plaid }` picks one option; spaces around each are trimmed.
//! - Choices nest: `{ a { red | blue } scarf | nothing }`.
//! - An empty option makes words optional: `a { | faded } photograph`.
//! - Choices sharing a tag pick the same position, which keeps a sentence
//!   consistent: `{@who man | woman}` … `{@who his | her}`.
//! - `{{` and `}}` are literal braces.
//!
//! Parsing happens at load, so a bad brace is reported with the file's name;
//! rolling uses the game's dice, so a seed always gives the same words.

use std::collections::BTreeMap;
use std::fmt;

use serde::Deserialize;

use crate::dice::Dice;

/// A piece of text that may hold choices, parsed and checked.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(try_from = "String")]
pub struct Flavor(Vec<Piece>);

#[derive(Clone, Debug, PartialEq)]
enum Piece {
    Text(String),
    Choice(Choice),
}

#[derive(Clone, Debug, PartialEq)]
struct Choice {
    /// Choices with the same tag all pick the same option index.
    tag: Option<String>,
    options: Vec<Flavor>,
}

/// A markup mistake, worded for someone editing a data file.
#[derive(Debug, PartialEq)]
pub struct FlavorError(String);

impl fmt::Display for FlavorError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl TryFrom<String> for Flavor {
    type Error = FlavorError;
    fn try_from(src: String) -> Result<Self, FlavorError> {
        Flavor::parse(&src)
    }
}

impl Flavor {
    /// Parse markup, checking braces and that linked choices line up.
    pub fn parse(src: &str) -> Result<Self, FlavorError> {
        let mut parser = Parser { src, pos: 0 };
        let flavor = parser.sequence(false)?.trimmed();
        let mut arity = BTreeMap::new();
        flavor.check_tags(&mut arity)?;
        Ok(flavor)
    }

    /// True when every roll would come out empty or whitespace.
    pub fn is_blank(&self) -> bool {
        self.0.iter().all(|p| match p {
            Piece::Text(t) => t.trim().is_empty(),
            Piece::Choice(c) => c.options.iter().all(Flavor::is_blank),
        })
    }

    /// Pick an option for every choice and return the finished text.
    pub fn roll(&self, dice: &mut Dice) -> String {
        let mut out = String::new();
        self.write(dice, &mut BTreeMap::new(), &mut out);
        tidy(&out)
    }

    fn write(&self, dice: &mut Dice, picks: &mut BTreeMap<String, usize>, out: &mut String) {
        for piece in &self.0 {
            match piece {
                Piece::Text(t) => out.push_str(t),
                Piece::Choice(c) => {
                    let len = c.options.len();
                    let i = match &c.tag {
                        Some(tag) => *picks.entry(tag.clone()).or_insert_with(|| dice.index(len)),
                        None => dice.index(len),
                    };
                    c.options[i].write(dice, picks, out);
                }
            }
        }
    }

    /// Every use of a tag must offer the same number of options, or a link
    /// could point past the end.
    fn check_tags(&self, arity: &mut BTreeMap<String, usize>) -> Result<(), FlavorError> {
        for piece in &self.0 {
            let Piece::Choice(c) = piece else { continue };
            if let Some(tag) = &c.tag {
                let first = *arity.entry(tag.clone()).or_insert(c.options.len());
                if first != c.options.len() {
                    let n = c.options.len();
                    return Err(FlavorError(format!(
                        "`@{tag}` has {first} options in one place and {n} in another; linked choices must match"
                    )));
                }
            }
            for option in &c.options {
                option.check_tags(arity)?;
            }
        }
        Ok(())
    }

    /// Drop whitespace at either end, so `{ red | blue }` gives `red`.
    fn trimmed(mut self) -> Self {
        if let Some(Piece::Text(t)) = self.0.first_mut() {
            *t = t.trim_start().to_string();
        }
        if let Some(Piece::Text(t)) = self.0.last_mut() {
            *t = t.trim_end().to_string();
        }
        self.0.retain(|p| !matches!(p, Piece::Text(t) if t.is_empty()));
        self
    }
}

/// Clean up after empty options: `a  photograph` and `rock ,` read wrong.
fn tidy(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.trim().chars() {
        let after_space = out.ends_with(' ');
        if c == ' ' && after_space {
            continue;
        }
        if after_space && matches!(c, ',' | '.' | ';' | ':' | '!' | '?') {
            out.pop();
        }
        out.push(c);
    }
    out
}

struct Parser<'a> {
    src: &'a str,
    pos: usize,
}

impl Parser<'_> {
    fn rest(&self) -> &str {
        &self.src[self.pos..]
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.rest().chars().next()?;
        self.pos += c.len_utf8();
        Some(c)
    }

    /// Text and choices up to the end, or (inside braces) up to the next
    /// `|` or `}`, which is left for the caller.
    fn sequence(&mut self, nested: bool) -> Result<Flavor, FlavorError> {
        let mut pieces = Vec::new();
        let mut text = String::new();
        while let Some(c) = self.rest().chars().next() {
            if (c == '{' || c == '}') && self.rest()[1..].starts_with(c) {
                self.pos += 2;
                text.push(c);
            } else if c == '{' {
                self.pos += 1;
                if !text.is_empty() {
                    pieces.push(Piece::Text(std::mem::take(&mut text)));
                }
                pieces.push(Piece::Choice(self.choice()?));
            } else if nested && (c == '|' || c == '}') {
                break;
            } else if c == '}' {
                return Err(FlavorError("`}` without a `{` before it (write `}}` for a literal brace)".into()));
            } else {
                self.bump();
                text.push(c);
            }
        }
        if !text.is_empty() {
            pieces.push(Piece::Text(text));
        }
        Ok(Flavor(pieces))
    }

    /// The inside of `{ … }`, just after the `{`.
    fn choice(&mut self) -> Result<Choice, FlavorError> {
        let tag = self.tag()?;
        let mut options = vec![self.sequence(true)?.trimmed()];
        loop {
            match self.bump() {
                Some('|') => options.push(self.sequence(true)?.trimmed()),
                Some('}') => break,
                _ => return Err(FlavorError("a `{` is never closed with `}`".into())),
            }
        }
        if options.len() < 2 {
            return Err(FlavorError("a `{ … }` needs two or more options split by `|`".into()));
        }
        Ok(Choice { tag, options })
    }

    /// An optional `@name` at the start of a choice.
    fn tag(&mut self) -> Result<Option<String>, FlavorError> {
        let skipped = self.rest().len() - self.rest().trim_start().len();
        if !self.rest()[skipped..].starts_with('@') {
            return Ok(None);
        }
        self.pos += skipped + 1;
        let name: String = self.rest().chars().take_while(|c| c.is_ascii_alphanumeric() || *c == '_').collect();
        if name.is_empty() {
            return Err(FlavorError("`@` must be followed by a name, like `{@who man | woman}`".into()));
        }
        self.pos += name.len();
        Ok(Some(name))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;

    fn rolls(src: &str) -> BTreeSet<String> {
        let f = Flavor::parse(src).unwrap();
        let mut dice = Dice::seeded(7);
        (0..200).map(|_| f.roll(&mut dice)).collect()
    }

    fn err(src: &str) -> String {
        Flavor::parse(src).unwrap_err().0
    }

    #[test]
    fn plain_text_passes_through() {
        assert_eq!(rolls("Just words."), BTreeSet::from(["Just words.".to_string()]));
    }

    #[test]
    fn picks_each_option_trimmed() {
        let want = ["a red scarf", "a blue scarf", "a plaid scarf"].map(String::from);
        assert_eq!(rolls("a { red | blue |plaid} scarf"), BTreeSet::from(want));
    }

    #[test]
    fn choices_nest() {
        let want = ["a red scarf", "a blue scarf", "nothing"].map(String::from);
        assert_eq!(rolls("{ a { red | blue } scarf | nothing }"), BTreeSet::from(want));
    }

    #[test]
    fn empty_options_leave_no_gaps() {
        let want = ["a photograph.", "a faded photograph.", "a photo, creased.", "a faded photo, creased."];
        let want = want.map(String::from);
        assert_eq!(rolls("a { | faded } photo{graph | , creased}."), BTreeSet::from(want));
        let want = ["Under a rock.", "Under a rock, wet."].map(String::from);
        assert_eq!(rolls("Under a rock { | , wet }."), BTreeSet::from(want));
    }

    #[test]
    fn linked_choices_agree() {
        let want = ["A man runs his hand", "A woman runs her hand"].map(String::from);
        assert_eq!(rolls("A {@who man | woman} runs {@who his | her} hand"), BTreeSet::from(want));
    }

    #[test]
    fn doubled_braces_are_literal() {
        assert_eq!(rolls("{{not a choice}}"), BTreeSet::from(["{not a choice}".to_string()]));
    }

    #[test]
    fn same_seed_same_words() {
        let f = Flavor::parse("{ a | b | c } { d | e | f } { g | h | i }").unwrap();
        let (mut d1, mut d2) = (Dice::seeded(3), Dice::seeded(3));
        assert_eq!(f.roll(&mut d1), f.roll(&mut d2));
    }

    #[test]
    fn reports_mistakes() {
        assert!(err("a { red | blue").contains("never closed"));
        assert!(err("red } blue").contains("without a `{`"));
        assert!(err("a { red } scarf").contains("two or more"));
        assert!(err("{@ a | b }").contains("followed by a name"));
        assert!(err("{@x a | b } {@x c | d | e }").contains("`@x` has 2 options"));
    }

    #[test]
    fn blank_means_nothing_to_say() {
        assert!(Flavor::parse("  { | }  ").unwrap().is_blank());
        assert!(!Flavor::parse("{ | x }").unwrap().is_blank());
    }
}
