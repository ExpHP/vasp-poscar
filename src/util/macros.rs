// Copyright 2018 Michael Lamparski
// Part of the vasp-poscar crate.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// general bail! and ensure! macros that don't constrain the type to failure::Error
macro_rules! g_bail { ($e:expr $(,)*) => { return Err($e.into()); }; }
macro_rules! g_ensure { ($cond:expr, $e:expr $(,)*) => { if !$cond { g_bail!($e); } }; }

/// Macro that generates a `[T; 3]` from a function on index.
///
/// ```rust
/// assert_eq!(
///     arr_3![i => 2*i],
///     [0, 2, 4],
/// );
/// ```
///
/// Implementing this utility as a macro rather than a callback-taking
/// function allows it to naturally support `?` expressions without having
/// to worry about type inference on the error type.
macro_rules! arr_3 {
    ($pat:pat => $expr:expr)
    => { [
        { let $pat = 0; $expr },
        { let $pat = 1; $expr },
        { let $pat = 2; $expr },
    ]};
}

macro_rules! mat_3 {
    (($row:pat, $col:pat) => $expr:expr)
    => { arr_3![$row => arr_3![$col => $expr]] };
}

macro_rules! zip {
    ($a:expr)
    => {{ $a.into_iter().map(|x| (x,)) }};

    ($a:expr, $b:expr)
    => {{ $a.into_iter().zip($b).map(|(a, b)| (a, b)) }};

    ($a:expr, $b:expr, $c: expr)
    => {{ $a.into_iter().zip($b).zip($c).map(|((a, b), c)| (a, b, c)) }};
}
