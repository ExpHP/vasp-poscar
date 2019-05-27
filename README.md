[![Crates.io](https://img.shields.io/crates/v/vasp-poscar.svg)](https://crates.io/crates/vasp-poscar)
[![Docs.rs](https://docs.rs/vasp-poscar/badge.svg)](https://docs.rs/vasp-poscar/*/vasp_poscar/)
[![Build Status](https://travis-ci.org/ExpHP/vasp-poscar.svg?branch=master)](https://travis-ci.org/ExpHP/vasp-poscar)

# POSCAR format for Rust

* [Release notes](doc/relnotes.md)
* [API Documentation](https://docs.rs/vasp-poscar/*/vasp_poscar/)

```toml
[dependencies]
vasp-poscar = "0.3.0"
```

A parser and printer for the POSCAR file format for representing crystallographic compounds.

POSCAR is primarily an input file format used by the [Vienna Ab initio Simulation Package (VASP)](https://www.vasp.at/), which has become fairly well-supported by a wide variety of software related to crystallography and molecular dynamics.

## Example

**An example file:**

```text
cubic diamond
  3.7
    0.5 0.5 0.0
    0.0 0.5 0.5
    0.5 0.0 0.5
   C
   2
Direct
  0.0 0.0 0.0
  0.25 0.25 0.25
```

**Example code:**

```rust
extern crate vasp_poscar;

use vasp_poscar::{Poscar, ScaleLine};

// read from a BufRead instance, such as &[u8] or BufReader<File>
let file = io::BufReader::new(File::open("POSCAR")?);
let poscar = Poscar::from_reader(file)?;

// get a RawPoscar object with public fields you can freely match on and manipulate
let mut poscar = poscar.into_raw();

assert_eq!(poscar.scale, ScaleLine::Factor(3.7));

poscar.comment = "[Subject Name Here] was here".into();
poscar.scale = ScaleLine::Volume(10.0);

// Turn back into a Poscar for writing.
// Notice Poscar implements Display.
let poscar = poscar.validate()?;
print!("{}, poscar);
```

## Notes about the format

### A birds-eye view

The POSCAR format is primarily equipped with two to three key pieces of information:

* A **periodic lattice**.
* A set of **coordinates** for sites in a unit cell.
* (From Vasp 5 onwards) The **elemental symbols** associated with the sites.

The structure in a POSCAR is *always* periodic in 3 dimensions. Lower-dimensional structures are represented approximately by assigning a long periodic length *(a "vacuum separation")* to the aperiodic directions. One can contrast this with XYZ files, which are only capable of representing aperiodic structures.

POSCAR also has some optional sections that are probably mostly really only used by VASP itself:

* Velocities
* Selective dynamics, which allows sites to be constrained to movement along a subset of the lattice vectors.
* **(TODO)** The... um... *"predictor corrector."*

### The nitty gritty

The format also unfortunately suffers a great deal from being, well... *underspecified.*

VASP's own documentation can be found here: http://cms.mpi.univie.ac.at/vasp/guide/node59.html.  Please refer to that page for questions regarding the purpose and semantics of each element in the file.

A somewhat fuller specification of the format's **syntax** (*as implemented by this crate*) can be found at [doc/format.md](doc/format.md).

## Scope of this crate

`vasp-poscar` is primarily a backend-level crate for *reading and writing a file format.*  It aims to provide:

* **reasonable diagnostics** on malformed files, with special consideration given to errors that are easy to make
* **round-trippable precision**; when a POSCAR is read in, it does not automatically absorb the scale into the lattice matrix, or convert everything into its favorite representation (direct vs cartesian). *Those are __your__ decisions to make!*

`vasp-poscar` is secondarily a crate for *managing the redundant forms of data that exist within the file.*  Notably:

* Obtaining the scaled lattice and *true* cartesian coordinates
* **(TODO)** Converting between direct and cartesian representations
* **(TODO)** Manipulating the scale and lattice with respect to each other (e.g. switching between scale and volume, or absorbing the scale into the lattice)

`vasp-poscar` is **not really a crate for doing science.**  It will *never* provide things like symmetry analysis, primitive structure search, supercell construction, perturbation of positions, or cutting across a plane, etc.  These things are simply not its job.

The expectation is that the data read by `vasp-poscar` may be used to construct an instance of a more versatile—and more opinionated—`Structure` type **implemented in another crate.**  (of course, if you are designing such a type, you are invited to depend on this crate as a parsing backend!)

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
