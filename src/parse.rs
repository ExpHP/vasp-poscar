// Copyright 2018 Michael Lamparski
// Part of the vasp-poscar crate.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use ::{Coords, RawPoscar, ScaleLine, Poscar};

use ::std::rc::Rc;
use ::std::io::prelude::*;
use ::std::ops::Range;
use ::std::str::FromStr;
use ::std::cmp::Ordering;
use ::std::path::{Path, PathBuf};

pub use self::error::ParseError;
mod error {
    use super::*;
    use ::std::fmt;

    #[derive(Debug, Fail)]
    pub struct ParseError {
        pub(crate) kind: Kind,
        pub(crate) path: Option<PathBuf>,
        // (NOTE: these are zero-based for maximum comfort, but the Display
        //        impl will use one-based indices for convention)
        pub(crate) line: Option<usize>,
        pub(crate) col: Option<usize>,
    }

    impl fmt::Display for ParseError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self.path.as_ref() {
                Some(p) => write!(f, "{}:", p.display())?,
                None => write!(f, "<input>:")?,
            }

            match (self.line, self.col) {
                (None, _) => {}
                (Some(r), None) => write!(f, "{}: ", r + 1)?,
                (Some(r), Some(c)) => write!(f, "{}:{}: ", r + 1, c + 1)?,
            }

            <Kind as fmt::Display>::fmt(&self.kind, f)
        }
    }

    use ::std::num::ParseFloatError;

    #[derive(Debug, Fail)]
    pub(crate) enum Kind {
        #[fail(display="{}", _0)] ParseFloat(ParseFloatError),
        #[fail(display="{}", _0)] ParseLogical(ParseLogicalError),
        #[fail(display="{}", _0)] ParseUnsigned(ParseUnsignedError),
        #[fail(display="{}", _0)] Generic(String),
    }

    impl From<ParseFloatError> for Kind { fn from(e: ParseFloatError) -> Kind { Kind::ParseFloat(e) } }
    impl From<ParseUnsignedError> for Kind { fn from(e: ParseUnsignedError) -> Kind { Kind::ParseUnsigned(e) } }
    impl From<ParseLogicalError> for Kind { fn from(e: ParseLogicalError) -> Kind { Kind::ParseLogical(e) } }
    impl<'a> From<&'a str> for Kind { fn from(e: &'a str) -> Kind { Kind::Generic(e.into()) } }
    impl From<String> for Kind { fn from(e: String) -> Kind { Kind::Generic(e) } }
}

// helper types for reading line by line.
// (NOTE: we could probably replace all this garbage with nom. Any takers?)
#[derive(Debug, Clone)]
pub(crate) struct Lines<I> {
    path: Option<Rc<PathBuf>>,
    cur: usize,
    lines: I,
}

// string with span info for errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Spanned<S=String> {
    path: Option<Rc<PathBuf>>,
    line: usize,
    col: usize,
    s: S,
}

impl<E, I> Lines<I>
where
    I: Iterator<Item=Result<String, E>>,
    E: ::failure::Fail,
{
    pub(crate) fn new<P: AsRef<Path>>(lines: I, path: Option<P>) -> Self
    { Self {
        path: path.map(|p| Rc::new(p.as_ref().to_owned())),
        lines,
        cur: 0,
    }}

    pub(crate) fn next(&mut self) -> Result<Spanned, ::failure::Error>
    {
        let path = self.path.clone();
        let line = self.cur;
        let col = 0;
        let s = self.lines.next().ok_or_else(|| {
            ParseError {
                kind: "unexpected end of file".into(),
                path: self.path.as_ref().map(|p| p.as_ref().to_owned()),
                line: Some(self.cur),
                col: None,
            }
        })??;

        self.cur += 1;
        Ok(Spanned { path, line, col, s })
    }
}

impl<S> Spanned<S> {
    pub(crate) fn error<K>(&self, kind: K) -> ParseError
    where K: Into<error::Kind>,
    { ParseError {
        kind: kind.into(),
        path: self.path.as_ref().map(|p| p.as_ref().to_owned()),
        line: Some(self.line),
        col: Some(self.col),
    }}
}

// NOTE: holdover until the method is stabilized on 1.24
fn is_ascii_whitespace(b: u8) -> bool {
    match b {
        b' ' | b'\t' | b'\r' | b'\n' => true,
        _ => false
    }
}

impl<S: AsRef<str>> Spanned<S> {
    pub(crate) fn as_str(&self) -> &str { self.s.as_ref() }

    pub(crate) fn slice(&self, range: Range<usize>) -> Spanned<&str>
    {
        Spanned {
            path: self.path.clone(),
            line: self.line,
            col: self.col + range.start,
            s: &self.s.as_ref()[range],
        }
    }

    // Like 's.trim().split_whitespace()', but tracks file position
    pub(crate) fn words<'a>(&'a self) -> Words<'a>
    {
        use ::std::iter::once;

        let bytes = self.s.as_ref().as_bytes().iter().cloned();

        // Pretend the word is surrounded by whitespace, and check each char with the previous.
        // The index corresponds to 'cur'.
        let prevs = once(b' ').chain(bytes.clone());
        let chars = bytes.clone().chain(once(b' '));
        let mut iter = prevs.zip(chars).enumerate();

        let mut out = vec![];

        'start:
        while let Some((start, (prev, cur))) = iter.next() {
            if is_ascii_whitespace(prev) && !is_ascii_whitespace(cur) {

                while let Some((end, (_, cur))) = iter.next() {
                    if is_ascii_whitespace(cur) {
                        out.push(self.slice(start..end));
                        continue 'start;
                    }
                }

                panic!("never encountered whitespace!");
            }
        }
        Words {
            path: self.path.clone(),
            line: self.line,
            iter: Box::new(out.into_iter()),
        }
    }

    pub(crate) fn parse<T>(&self) -> Result<T, ParseError>
    where T: FromStr,
          T::Err: Into<error::Kind>,
    { self.s.as_ref().parse().map_err(|e| self.error(e)) }


    // The meaningful character for a flag line. It's the first character, PERIOD.
    // Even if that character is whitespace!
    // (mind: since 'Lines' omits the line terminators, this *can* produce None)
    pub(crate) fn control_char(&self) -> Option<char> { self.as_str().chars().next() }
}

pub(crate) struct Words<'a> {
    path: Option<Rc<PathBuf>>,
    line: usize,
    iter: Box<Iterator<Item=Spanned<&'a str>> + 'a>,
}

impl<'a> Iterator for Words<'a> {
    type Item = Spanned<&'a str>;
    fn next(&mut self) -> Option<Self::Item> { self.iter.next() }
}

impl<'a> Words<'a> {
    pub(crate) fn next_or_err(&mut self, msg: &str) -> Result<Spanned<&'a str>, ParseError>
    { self.next().ok_or_else(|| ParseError {
        kind: msg.into(),
        path: self.path.as_ref().map(|p| p.as_ref().to_owned()),
        line: Some(self.line),
        col: None,
    })}
}

#[test]
fn words() {
    // test with space at boundaries
    let s = Spanned { path: None, line: 0, col: 0, s: "  aa b   ccc  " };
    assert_eq!(
        s.words().collect::<Vec<_>>(),
        vec![
            Spanned { path: None, line: 0, col: 2, s: "aa" },
            Spanned { path: None, line: 0, col: 5, s: "b" },
            Spanned { path: None, line: 0, col: 9, s: "ccc" },
        ],
    );

    // test nonzero col, and words at boundaries
    let s = s.slice(3..s.as_str().len() - 3);
    assert_eq!(
        s.words().collect::<Vec<_>>(),
        vec![
            Spanned { path: None, line: 0, col: 3, s: "a" },
            Spanned { path: None, line: 0, col: 5, s: "b" },
            Spanned { path: None, line: 0, col: 9, s: "cc" },
        ],
    );
}

// ------------------

// Parses the way that Fortran's read(*) does when reading into a LOGICAL.
// Spec: https://docs.oracle.com/cd/E19957-01/805-4939/6j4m0vnc5/index.html
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) struct Logical(pub bool);

#[derive(Debug, Fail)]
#[fail(display = "invalid Fortran logical value: {:?}", _0)]
pub struct ParseLogicalError(String);

impl FromStr for Logical {
    type Err = ParseLogicalError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let mut s = input.as_bytes();

        // An optional dot...
        if s.get(0) == Some(&b'.') {
            s = &s[1..];
        }

        // ...followed by a single case-insensitive character. The rest is ignored.
        match s.get(0) {
            Some(&b't') | Some(&b'T') => Ok(Logical(true)),
            Some(&b'f') | Some(&b'F') => Ok(Logical(false)),
            _ => Err(ParseLogicalError(input.to_string())),
        }
    }
}

// Parses like u64 but forbids the leading '+'.
//
// Mentioned under 'primitives' in the file format doc page.
// TODO: link
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) struct Unsigned(pub u64);

#[derive(Debug, Fail)]
#[fail(display = "{}", _0)]
pub(crate) struct ParseUnsignedError(::failure::Error);

#[derive(Debug, Fail)]
#[fail(display = "invalid digit for integer")]
pub(crate) struct LeadingPlusError;

impl FromStr for Unsigned {
    type Err = ParseUnsignedError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input.chars().next() {
            Some('+') => g_bail!(ParseUnsignedError(LeadingPlusError.into())),
            _ => {},
        }
        input.parse().map(Unsigned).map_err(|e| ParseUnsignedError(e.into()))
    }
}

macro_rules! arr_3 {
    ($pat:pat => $expr:expr)
    => { [
        { let $pat = 0; $expr },
        { let $pat = 1; $expr },
        { let $pat = 2; $expr },
    ]}
}

/// Reads a POSCAR from an open file.
///
/// This form is unable to include a filename in error messages.
// FIXME how do other libraries handle this?
//       maybe the filename is simply not this crate's responsibility?
pub fn from_reader<R>(f: R) -> Result<Poscar, ::failure::Error>
where R: BufRead,
{ _from_reader(f, None::<PathBuf>) }

/// Reads a POSCAR from a filesystem path.
pub fn from_path<P>(path: P) -> Result<Poscar, ::failure::Error>
where P: AsRef<Path>,
{
    let f = ::std::fs::File::open(path.as_ref())?;
    let f = ::std::io::BufReader::new(f);
    _from_reader(f, Some(path))
}

fn parse_unsigned(s: &str) -> Result<u64, ParseUnsignedError>
{ let Unsigned(x) = s.parse()?; Ok(x) }

fn _from_reader<R, P>(f: R, path: Option<P>) -> Result<Poscar, ::failure::Error>
where R: BufRead, P: AsRef<Path>,
{
    let mut lines = Lines::new(f.lines(), path);

    let comment = lines.next()?.as_str().to_string();

    let scale;
    {
        let line = lines.next()?;
        let mut words = line.words();

        // First word is the scale factor.
        let word = words.next_or_err("expected scale")?;
        let value: f64 = word.parse()?;

        scale = match value.partial_cmp(&0.0) {
            Some(Ordering::Less) => ScaleLine::Volume(-value),
            Some(Ordering::Greater) => ScaleLine::Factor(value),
            Some(Ordering::Equal) => bail!(word.error("scale cannot be zero")),
            None => bail!(word.error("scale cannot be nan")),
        };

        // In the vasp 5.4.1 source code there is an undocumented(?) "feature":
        // If the number of (whitespace-separated?) tokens at the beginning of the line
        // that succesfully parse as floats is exactly 3, then they are regarded as
        // scales for each of the (cartesian) XYZ axes. (note: axes, not lattice vectors!)
        //
        // The existence of this feature is not acknowledged by either ASE or pymatgen,
        // and in fact, in the version I'm looking at, not even VASP handles it properly!
        // (the scales are not taken into account when generating CONTCAR)
        // It seems fair to say that nobody will ever use this broken feature on purpose.
        //
        // Meanwhile, forgetting the scale line is an easy mistake, and coincidentally
        // puts three floats in that location. This pretty much always generates an error
        // *somewhere*, but sometimes it can be far away from this line.
        //
        // For these reasons, we'll generate an error when there are two or more floats.
        if let Some(word) = words.next() {
            if let Ok(_) = word.parse::<f64>() {
                bail!(word.error("too many floats on scale line (expected just one)"));
            }
        }
    };

    let lattice_vectors = arr_3![_ => {
        let line = lines.next()?;
        let mut words = line.words();
        arr_3![_ => {
            words.next_or_err("expected three components for lattice vector")?.parse()?
        }]
        // rest is freeform comment
    }];

    // symbols and counts
    let (group_symbols, group_counts, n) = {
        let line = lines.next()?;

        // (make sure there is a non-whitespace char) TODO test case
        let _ = line.words().next_or_err("expected at least one element or count")?;

        // New in vasp 5, a line with elemental symbols can appear before the line with counts.
        let (group_symbols, counts_line) = match line.as_str().trim().as_bytes()[0] {
            // this line clearly has counts
            b'0'...b'9' => (None, line),

            // this line must have symbols
            _ => {
                let kinds = line.words().map(|s| s.as_str().to_string()).collect::<Vec<_>>();
                (Some(kinds), lines.next()?)
            },
        };

        let group_counts: Result<Vec<usize>, _> = {
            counts_line.words().map(|s| parse_unsigned(s.as_str()).map(|x| x as usize))
                               .take_while(|e| e.is_ok())
                               .collect()
        };
        let group_counts = group_counts?;

        if let Some(ref group_symbols) = group_symbols {
            if group_symbols.len() != group_counts.len() {
                bail!(counts_line.error("Inconsistent number of counts"));
            }
        }

        let n = group_counts.iter().sum();
        if n == 0 {
            bail!(counts_line.error("There must be at least one atom."));
        }

        (group_symbols, group_counts, n)
    };

    // flags
    let (has_direct, has_selective_dynamics);
    {
        let line = lines.next()?;

        let line = match line.control_char() {
            Some('s') |
            Some('S') => { has_selective_dynamics = true; lines.next()? },
            _ => { has_selective_dynamics = false; line },
        };

        has_direct = match line.control_char() {
            Some('c') | Some('C') | Some('k') | Some('K') => false,

            // Technically speaking, according to the VASP docs, *anything else*
            // should fall under the umbrella of direct coordinates....
            // But a line like "  Cartesian" is just too damn suspicious.
            Some(' ') if !line.as_str().trim().is_empty() => {
                // TODO: Log warning via log
                true
            },

            _ => true,
        };
        // rest is freeform comment
    };

    let (positions, dynamics) = {
        let mut positions = vec![];
        let mut dynamics = match has_selective_dynamics {
            true => Some(vec![]),
            false => None,
        };

        for _ in 0..n {
            let line = lines.next()?;
            let mut words = line.words();

            positions.push(arr_3![_ => words.next_or_err("expected 3 coordinates")?.parse()?]);

            if let Some(selective_dynamics) = dynamics.as_mut() {
                selective_dynamics.push({
                    arr_3![_ => {
                        words.next_or_err("expected 3 boolean flags")?.parse::<Logical>()?.0
                    }]
                })
            }
            // rest is freeform comment
        };

        (positions, dynamics)
    };

    let positions = match has_direct {
        true  => Coords::Frac(positions),
        false => Coords::Cart(positions),
    };

    let velocities = None; // FIXME

    // we don't support any other junk
    while let Ok(line) = lines.next() {
        ensure!(line.as_str().trim().is_empty(), line.error("expected EOF"));
    }

    Ok(RawPoscar {
        comment, scale, positions, lattice_vectors,
        group_symbols, group_counts, velocities, dynamics,
    }.validate().expect("an invariant was not checked during parsing (this is a bug!)"))
}
