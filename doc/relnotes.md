# `vasp-poscar` release notes

## **v0.3.1**:
* Added `Poscar::num_sites`, `Poscar::site_symbols`, and `Builder::site_symbols`.

## **v0.3.0**:
* Updated to the 2018 edition of Rust.
* The `Box<dyn VeclikeIterator>` return types of `Poscar::group_symbols` and `Poscar::group_counts` have been replaced with `impl VeclikeIterator`.
* Added `Poscar::scaled_positions`.

## **v0.2.0**:
* Added accessors for most immediately useful data to `Poscar`.
* Added `Builder` api to replace struct-syntax construction of `RawPoscar`.
* Renamed `Poscar::raw` to `into_raw`

## **v0.1.1**:
* Fix missing metadata for the crates.io page.

## **v0.1.0**:
* **doc/format.md**
* Parsing via `Poscar::from_reader`.
* Manipulation and construction via `RawPoscar`.
* Output via `std::fmt::Display`.
