# POSCAR format

The POSCAR format is phenomenally underspecified, and there is a lot of false information about it on the web.  This section aims to clarify the crate author's interpretation of the format, based on reviewing the behavior of other libraries for VASP interop, and checking against the VASP 5.4.1 implementation.

**IMPORTANT: This document only reflects the current implementation of the `vasp-poscar` crate.**  Some of these decisions may be revisited in future versions of the crate, and some inputs that used to parse may stop parsing, or vice versa.  If you want to ensure that your files are compatible with all future versions of the crate, my only advice for now is to apply your common sense, and to always anticipate the worst outcome possible (as you surely already *have* been doing when using crystallographic software!).

## Structural elements

This section is an attempt to describe key building blocks of the POSCAR format.

### Primitives

Most lines in the format are specified to contain one or more **primitives,** such as integers or reals.

The VASP documentation does not document what it expects most primitives to look like, leaving every implementation of the format to fend for itself. Needless to say, no two implementations are alike.

In actuality, the implementation of VASP uses FORTRAN's `read(*)` for almost all of its parsing. As a result, the *actual* set of inputs accepted by VASP is far, far greater than what most might expect, allowing things such as optional commas between fields, "0.0*3" for repetitions, or ".tiddlyWinks" as a selective dynamics flag.

But there is no compelling reason for this crate to support all of these intricacies when nobody will use them.  Therefore, this crate defines the format of each primitive as follows:

* All primitives are understood to be **separated by spaces or tabs**. The rest of `read(*)`'s wild syntax is not supported.
* A line containing primitives may optionally begin with leading whitespace and end with trailing whitespace.
* An **unsigned integer** is whatever can be parsed using `<u64 as std::str::FromStr>`, with the additional constraint that it may not have a leading `+`. (this constraint makes the specification of the counts/symbols lines simpler)
* A **real** is whatever can be parsed using `<f64 as std::str::FromStr>`.
* A **logical** is parsed [like `read(*)` does](https://docs.oracle.com/cd/E19957-01/805-4939/6j4m0vnc5/index.html), which basically appears to amount to the regex `\.?[tTfF].*`.

For best compatibility with other low-quality implementations, you would be wise to follow the following limitations:

* Please do not prefix integers with a leading zero.  (other implementations may regard this as octal)
* Please only use single capital letters (`T` and `F`) for logicals.

### "Freeform comments"

Let us define a comment as *any arbitrary freeform text at the end of a line after the parts that VASP actually cares about.* If that definition terrifies you, it *should!*

If you look at some of the examples in VASP's very own documentation, you'll find plain english text right next to actual meaningful data (such as in the example input for everybody's favorite structure, "cubic&nbsp;diamond&nbsp;&nbsp;&nbsp;&nbsp;comment&nbsp;line"). Needless to say, the `poscar` crate makes large sacrifices in terms of diagnostic quality in order to be able to parse these files.

But that shouldn't surprise you. This describes virtually every VASP compatibility library ever. All this is merely justification for why this crate is so seemingly tolerant of malformed input.

### Flag lines

A "flag line" is one whose **very first character** (deemed the control character) is significant. **Spaces count. Do not indent these lines!**

A flag line can be empty. One could say the control character is some out-of-band value like `None` in this case.

*The rest of the line after the control character regarded is a freeform comment.*

## Structure

### The comment

The first line is known as **the comment**.  It can contain anything.

### Scale line

The scale line contains a single real. (reminder: see the section on [Primitives] for the rules of tokenization and accepted formats)  *Anything after this is a comment.*

Its sign is significant (a negative scale is interpreted as a target volume).  It may not be zero.

### Lattice lines

Three lines, each with three reals. *The rest of each line is a comment.*

### Symbols and counts

* Symbols line (optional)
* Counts line

```text
  Si O
  24 8  freeform comment
```

The optional symbols line before the counts line is detected by checking if the first non-whitespace character is a digit from `0` to `9`.

Every whitespace-separated token on the symbols line is regarded as a symbol; *this line has no freeform comment.*  A symbol is forbidden from beginning with a digit; however, beyond that, they are not validated as elemental symbols. (Knowing the periodic table is considered "out of scope" for this crate.)

Each whitespace-separacted word on the counts line *up until the first word which does not parse as an unsigned integer* (see the section on primitives) is regarded as a count. *The rest of the line is a freeform comment.*

If symbols are provided, the number of counts and symbols must match.

* It is forbidden for the total atom count to be zero. (this is in anticipation of eventual support for pymatgen-style symbols embedded in the coordinate data comments)
* It follows that the number of counts also must not be zero.
* It is *discouraged* for any of the individual counts to be zero.

### Position data and selective dynamics

* Selective dynamics line (optional)
* Coordinate system line
* Data lines

```text
Selective dynamics
Cartesian
  0.0 0.0 0.0 T T F
  0.2 0.2 0.2 T T T
```

The first is an optional [flag line] whose control character is `S` or `s`.
The second flag line is interpreted as follows:
* A control character in the string `"cCkK"` means cartesian coords.
* Anything else means direct coordinates.
  * This includes an empty line. (the CONTCAR file produced by VASP actually does this!!)
  * This even includes a line like `"   Cartesian"`. I am so, so sorry. The most this crate can do in such cases is to produce a warning via `log!`.

<!--
(FIXME use log)
(FIXME link all the things)
-->

Each data line begins with **three reals**. If Selective dynamics is enabled, then these are followed by **three logicals**. *The rest of the line is a comment.*

As stated earlier, this crate parses logicals using the grammar of Fortran's `read(*)`. It will accept input such as `"T"`, `"f"`, `".TRUE."` or `".T"`. When writing files it will always print `"T"` or `"F"`.

It is strongly recommended that you always **use** `T` and `F` as well, for greatest compatibility with other libraries. In a brief review of other implementations, it was found that both ASE and pymatgen parse these flags in dangerous ways that make absolutely no attempt to validate their assumptions about the input.

### Velocities (optional)

* Coordinate system line (sometimes misunderstood to be simply a blank line, thanks to the example set by CONTCAR)
* Data lines

These may optionally appear immediately after the last coordinate data line. The first line is a flag line just like the one for positions.  Each data line has three reals. *The rest is a freeform comment.*

**Note:** When VASP itself writes a CONTCAR file, **it writes a blank line for the coordinate system line.**  Because it does not start with "c" or "k", this blank line in fact indicates that the velocities are in **direct coordinates**.  However, some libraries (such as pymatgen) actually expect this line to always be blank; therefore, *this crate also chooses to write a blank line when the velocities are in direct units.*

### Trailing blank lines

Trailing lines at the end of the file are allowed as long as they contain nothing but whitespace.

The astute may notice that, together with the very real fact that the control line for velocities may be a blank line, it would be impossible to tell whether or not velocities are "present" for a structure with zero atoms.  Fortunately, such a structure is already forbidden.
