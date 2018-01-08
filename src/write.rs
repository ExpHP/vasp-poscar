// Copyright 2018 Michael Lamparski
// Part of the vasp-poscar crate.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use ::std::fmt;
use ::{Poscar, RawPoscar, ScaleLine, Coords};

impl fmt::Display for Poscar {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    { ::write::display(f, self) }
}

fn display(w: &mut fmt::Formatter, poscar: &Poscar) -> fmt::Result
{
    let &Poscar(RawPoscar {
        scale, ref lattice_vectors, ref velocities, ref dynamics,
        ref comment, ref positions, ref group_counts, ref group_symbols,
    }) = poscar;

    assert!(!comment.contains("\n"), "BUG");
    assert!(!comment.contains("\r"), "BUG");

    writeln!(w, "{}", comment)?;
    match scale {
        ScaleLine::Factor(x) => writeln!(w, "  {}", Dtoa(x))?,
        ScaleLine::Volume(x) => writeln!(w, "  -{}", Dtoa(x))?,
    }

    for row in lattice_vectors {
        writeln!(w, "    {}", By3(*row, Dtoa))?;
    }

    if let Some(group_symbols) = group_symbols.as_ref() {
        write!(w, "  ")?;
        write_sep(&mut *w, " ", group_symbols.iter().map(|s| format!("{:>2}", s)))?;
        writeln!(w)?;
    }

    assert!(!group_counts.is_empty(), "BUG");
    write!(w, "  ")?;
    write_sep(&mut *w, " ", group_counts.iter().map(|&c| format!("{:>2}", c)))?;
    writeln!(w)?;

    if let &Some(_) = dynamics {
        writeln!(w, "Selective Dynamics")?;
    }

    match positions {
        &Coords::Cart(_) => writeln!(w, "Cartesian")?,
        &Coords::Frac(_) => writeln!(w, "Direct")?,
    }

    let positions = positions.as_ref().raw();
    for (i, pos) in positions.iter().enumerate() {
        write!(w, "  {}", By3(*pos, Dtoa))?;
        if let &Some(ref dynamics) = dynamics {
            let fmt = |b| match b { true => 'T', false => 'F' };
            write!(w, " {}", By3(dynamics[i], fmt))?;
        }
        writeln!(w)?;
    }

    if let &Some(ref velocities) = velocities {
        match velocities {
            &Coords::Cart(_) => writeln!(w, "Cartesian")?,
            // (NOTE: typical appearance in CONTCAR; pymatgen expects this form)
            &Coords::Frac(_) => writeln!(w, "")?,
        }

        let velocities = velocities.as_ref().raw();
        for v in velocities {
            writeln!(w, "  {}", By3(*v, Dtoa))?;
        }
    }

    Ok(())
}

fn write_sep<W, Xs>(mut w: W, sep: &str, xs: Xs) -> fmt::Result
where
    W: fmt::Write,
    Xs: IntoIterator,
    Xs::Item: fmt::Display,
{
    let mut xs = xs.into_iter();
    if let Some(x) = xs.next() {
        write!(&mut w, "{}", x)?;
    }
    for x in xs {
        write!(&mut w, "{}{}", sep, x)?;
    }
    Ok(())
}

struct Dtoa(f64);
impl fmt::Display for Dtoa {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // not the most efficient thing in the world...
        let mut bytes = vec![];
        ::dtoa::write(&mut bytes, self.0).map_err(|_| fmt::Error)?;
        f.write_str(&String::from_utf8(bytes).unwrap())
    }
}

// Formats three space-separated tokens after applying a conversion function to each.
// Merely having this around makes it easier to remember to use Dtoa.
struct By3<A, F>([A; 3], F);
impl<A, B, F> fmt::Display for By3<A, F>
where A: Clone,
      F: Fn(A) -> B,
      B: fmt::Display,
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let By3(ref arr, ref f) = *self;
        write!(fmt, "{} {} {}", f(arr[0].clone()), f(arr[1].clone()), f(arr[2].clone()))
    }
}
