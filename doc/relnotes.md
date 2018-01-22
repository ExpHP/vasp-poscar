# `vasp-poscar` release notes

## **v0.2.0**:
* Added accessors for most immediately useful data to `Poscar`.
* Added `Builder` api to replace struct-syntax construction of `RawPoscar`.

## **v0.1.1**:
* Fix missing metadata for the crates.io page.

## **v0.1.0**:
* **doc/format.md**
* Parsing via `Poscar::from_reader`.
* Manipulation and construction via `RawPoscar`.
* Output via `std::fmt::Display`.
