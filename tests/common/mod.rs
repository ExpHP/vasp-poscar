macro_rules! assert_matches {
    ($pat:pat $(if $cond:expr)*, $expr:expr $(,)*) => {{
        let e = $expr;
        match e {
            $pat $(if $cond)* => {},
            _ => panic!("assert_matches failed!
Expected: {}
  Actual: {:#?}", stringify!($pat), e),
        }
    }}
}
