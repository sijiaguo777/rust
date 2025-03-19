/// Returns a `String` object with leading and trailing whitespace.
/// 
/// # Returns
/// 
/// A `String` object containing "  A String object  ".
fn ret_string() -> String {
    String::from("  A String object  ")
}

/// Chooses between two string slices based on a boolean flag.
/// 
/// # Parameters
/// 
/// - `s1`: The first string slice.
/// - `s2`: The second string slice.
/// - `select_s1`: A boolean flag indicating which string slice to choose.
/// 
/// # Returns
/// 
/// The chosen string slice, either `s1` or `s2`.
fn choose_str<'a>(s1: &'a str, s2: &'a str, select_s1: bool) -> &'a str {
    if select_s1 { 
        s1 
    } else { 
        s2
    }
}


enum OOR<'a> {
    Owned(String),
    Borrowed(&'a str),
}

use std::ops::{Deref,DerefMut};

impl Deref for OOR<'_> {
    type Target = str;

    /// Dereferences the `OOR` enum to a string slice.
    /// 
    /// # Returns
    /// 
    /// A string slice reference to the inner `String` or borrowed `str`.
    fn deref(&self) -> &Self::Target {
        match self {
            OOR::Owned(s) => s, 
            OOR::Borrowed(s) => s, 
        }
    }
}

// mutaboe reference to a string slice in a borrowed string
impl DerefMut for OOR<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // if the string is borrowed, convert it to owned in order to mutate and store a String before you can hand out a &mut str.
        if let OOR::Borrowed(s) = self { 
            *self = OOR::Owned(s.to_string());
        }
        match self {
            OOR::Owned(ref mut s) => s,
            _ => unreachable!(),
        }
    }
}


fn main() {
    let binding = ret_string();
    let s = binding.trim();
    assert_eq!(s, "A String object");

    // Check Deref for both variants of OOR
    let s1 = OOR::Owned(String::from("  Hello, world.  "));
    assert_eq!(s1.trim(), "Hello, world.");
    let mut s2 = OOR::Borrowed("  Hello, world!  ");
    assert_eq!(s2.trim(), "Hello, world!");

    // Check choose
    let s = choose_str(&s1, &s2, true);
    assert_eq!(s.trim(), "Hello, world.");
    let s = choose_str(&s1, &s2, false);
    assert_eq!(s.trim(), "Hello, world!");

    // Check DerefMut, a borrowed string should become owned
    assert!(matches!(s1, OOR::Owned(_)));
    assert!(matches!(s2, OOR::Borrowed(_)));
    unsafe {
        for c in s2.as_bytes_mut() {
            if *c == b'!' {
                *c = b'?';
            }
        }
    }
    assert!(matches!(s2, OOR::Owned(_)));
    assert_eq!(s2.trim(), "Hello, world?");
}

