#[macro_export]
macro_rules! tuples {
    // Entry point
    ($macro_name:ident! $($Types: ident)*) => {
        $macro_name!();
        $crate::tuples!($macro_name! ; $($Types)*);
    };
    // Process
    ($macro_name:ident! $($Before: ident)*; $This:ident $($Rest: ident)*) => {
        $macro_name!($($Before)* $This);
        $crate::tuples!($macro_name! $($Before)* $This; $($Rest)*);
    };
    // Exit point
    ($macro_name:ident! $($Before: ident)*;) => {};
}
