//! The Kotlin binding and the JNI exports are one contract: every `external
//! fun` in `WaterUiNative.kt` has a `Java_..._WaterUiNative_<name>` export
//! taking the same number of arguments (after `env` and `class`), and the
//! other way round. A rename on one side alone fails here, not on a phone.

use std::collections::BTreeMap;

const KOTLIN: &str = include_str!("../kotlin/io/github/ryceg/waterui/WaterUiNative.kt");
const RUST: &str = include_str!("../src/jni_api.rs");
const PREFIX: &str = "Java_io_github_ryceg_waterui_WaterUiNative_";

/// The text between the parentheses that open at `from`.
fn parenthesised(text: &str, from: usize) -> &str {
    let open = from + text[from..].find('(').expect("an argument list");
    let mut depth = 0;
    for (i, c) in text[open..].char_indices() {
        match c {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return &text[open + 1..open + i];
                }
            }
            _ => {}
        }
    }
    panic!("unbalanced parentheses");
}

fn arity(args: &str) -> usize {
    args.split(',').filter(|a| !a.trim().is_empty()).count()
}

fn kotlin_functions() -> BTreeMap<String, usize> {
    KOTLIN
        .match_indices("external fun ")
        .map(|(at, m)| {
            let rest = &KOTLIN[at + m.len()..];
            let name: String = rest.chars().take_while(|c| c.is_alphanumeric()).collect();
            (name, arity(parenthesised(KOTLIN, at)))
        })
        .collect()
}

fn rust_exports() -> BTreeMap<String, usize> {
    RUST.match_indices(PREFIX)
        .map(|(at, m)| {
            let rest = &RUST[at + m.len()..];
            let name: String = rest.chars().take_while(|c| c.is_alphanumeric()).collect();
            // Minus `env` and `class`.
            (name, arity(parenthesised(RUST, at)) - 2)
        })
        .collect()
}

#[test]
fn every_external_fun_has_an_export_of_the_same_arity() {
    let kotlin = kotlin_functions();
    let rust = rust_exports();
    assert!(
        kotlin.len() > 20,
        "found only {} external funs",
        kotlin.len()
    );
    assert_eq!(kotlin, rust);
}

#[test]
fn the_exception_class_the_exports_throw_is_the_bindings() {
    assert!(RUST.contains("\"io/github/ryceg/waterui/WaterUiException\""));
    assert!(KOTLIN.contains("class WaterUiException(message: String)"));
}
