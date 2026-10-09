// The test every crate's error conversion shares, included into each
// crate's `error/tests.rs` with `include!`: a crate's tests cannot import
// another crate's test code, and the facade re-exports nothing for tests.

/// Writes a crate error's conversion test.
///
/// It writes the type `$converted`, which records, for each method of
/// `$trait`, the method's name, what it was given as `$held`, and, unless
/// the trait's methods take `without message`, the message, and the test
/// `$test`, which converts each error of `$cases` through `$from`, by
/// reference when `&` is given, and finds the method of its variant, what
/// the variant holds, and the error's own message. `$setup` binds what the
/// cases and their held text name.
macro_rules! conversion_test {
    (
        $(#[$doc:meta])*
        fn $test:ident: $error:ty => $converted:ident through $trait:ident::$from:ident($($by:tt)?) {
            methods {
                $( $method:ident($( $argument:ident: $type:ty ),*) => $held:expr; )*
            }
            setup { $($setup:tt)* }
            cases {
                $( $case:expr => $case_method:ident, $case_held:expr; )*
            }
        }
    ) => {
        /// An error as a crate outside this one converts it: the method that
        /// built the value, what the variant held, and the message.
        #[derive(Debug, PartialEq, Eq)]
        struct $converted {
            method: &'static str,
            held: String,
            message: String,
        }

        impl $trait for $converted {
            $(
                fn $method($($argument: $type,)* message: String) -> Self {
                    Self {
                        method: stringify!($method),
                        held: $held,
                        message,
                    }
                }
            )*
        }

        $(#[$doc])*
        #[test]
        fn $test() {
            let convert = |error: $error| $converted::$from($($by)? error);
            $($setup)*
            $(
                {
                    let error: $error = $case;
                    let message = error.to_string();
                    assert_eq!(
                        convert(error),
                        $converted {
                            method: stringify!($case_method),
                            held: $case_held,
                            message,
                        }
                    );
                }
            )*
        }
    };
    (
        $(#[$doc:meta])*
        fn $test:ident: $error:ty => $converted:ident through $trait:ident::$from:ident($($by:tt)?) without message {
            methods {
                $( $method:ident($( $argument:ident: $type:ty ),*) => $held:expr; )*
            }
            setup { $($setup:tt)* }
            cases {
                $( $case:expr => $case_method:ident, $case_held:expr; )*
            }
        }
    ) => {
        /// A value as a crate outside this one converts it: the method that
        /// built it and what the variant held.
        #[derive(Debug, PartialEq, Eq)]
        struct $converted {
            method: &'static str,
            held: String,
        }

        impl $trait for $converted {
            $(
                fn $method($($argument: $type),*) -> Self {
                    Self {
                        method: stringify!($method),
                        held: $held,
                    }
                }
            )*
        }

        $(#[$doc])*
        #[test]
        fn $test() {
            let convert = |error: $error| $converted::$from($($by)? error);
            $($setup)*
            $(
                {
                    let error: $error = $case;
                    assert_eq!(
                        convert(error),
                        $converted {
                            method: stringify!($case_method),
                            held: $case_held,
                        }
                    );
                }
            )*
        }
    };
}
