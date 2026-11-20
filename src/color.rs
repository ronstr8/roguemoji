//! Colors written in data files: `#RRGGBB`, or a name from the X11 color
//! list (`firebrick`, `light goldenrod`, `gray50`).
//!
//! `rgb.txt` is the X.Org list (from x11-common), embedded so names work on
//! every platform, including ones without X.

const NAMES: &str = include_str!("rgb.txt");

/// Red, green and blue, 0-255.
pub type Rgb = (u8, u8, u8);

/// A color from a data file: hex, or an X11 name matched ignoring case and spaces.
pub fn parse(text: &str) -> Option<Rgb> {
    if text.starts_with('#') { hex(text) } else { named(text) }
}

/// `#RRGGBB` as red, green and blue.
fn hex(text: &str) -> Option<Rgb> {
    let digits = text.strip_prefix('#').filter(|h| h.len() == 6 && h.is_ascii())?;
    let part = |i: usize| u8::from_str_radix(&digits[i..i + 2], 16).ok();
    Some((part(0)?, part(2)?, part(4)?))
}

/// A name from the X11 list. Its lines read `R G B<tab(s)>name`; `!` starts a comment.
fn named(name: &str) -> Option<Rgb> {
    let wanted = squash(name);
    NAMES.lines().filter(|l| !l.starts_with('!')).find_map(|line| {
        let mut fields = line.split_whitespace();
        let mut channel = || fields.next()?.parse::<u8>().ok();
        let rgb = (channel()?, channel()?, channel()?);
        (squash(&fields.collect::<String>()) == wanted).then_some(rgb)
    })
}

/// Lowercase with spaces removed, so `Light Goldenrod` matches `LightGoldenrod`.
fn squash(name: &str) -> String {
    name.chars().filter(|c| !c.is_whitespace()).flat_map(char::to_lowercase).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_hex() {
        assert_eq!(parse("#C03a00"), Some((0xc0, 0x3a, 0x00)));
        assert_eq!(parse("#c03a0g"), None);
        assert_eq!(parse("#c03a"), None);
    }

    #[test]
    fn reads_x11_names_ignoring_case_and_spaces() {
        assert_eq!(parse("firebrick"), Some((178, 34, 34)));
        assert_eq!(parse("Light Goldenrod"), Some((238, 221, 130)));
        assert_eq!(parse("lightgoldenrod"), Some((238, 221, 130)));
        assert_eq!(parse("gray50"), Some((127, 127, 127)));
        assert_eq!(parse("no such color"), None);
        assert_eq!(parse(""), None);
    }
}
