#![allow(
    non_camel_case_types,
    non_snake_case,
    clippy::pub_underscore_fields,
    clippy::style,
    clippy::empty_structs_with_brackets,
    clippy::too_many_arguments,
    clippy::type_complexity,
    missing_docs,
    dead_code
)]

// Counter contract bindings generated at compile time from ABI
alloy::sol! {
    #[sol(rpc)]
    Counter,
    "abis/Counter.json"
}
