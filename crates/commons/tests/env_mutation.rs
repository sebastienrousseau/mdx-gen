//! Integration tests for the `env` module that require
//! mutating process-wide environment variables. The library
//! crate forbids `unsafe_code`, so `env::set_var` (which is
//! `unsafe` on Rust 1.88+) has to live in a separate
//! compilation unit. The workspace lints file denies
//! `unsafe_code` (not forbids), so we can locally override it
//! here where it is genuinely required for env mutation.

#![allow(unsafe_code)]

use std::env;
use std::sync::{Mutex, OnceLock};

use commons::env::{
    EnvConfig, EnvError, get_bool, get_environment, get_list, get_string, is_development,
    is_production, is_set, is_test, require_env, try_get_env,
};

/// Serialize env mutations: other tests in this file could race
/// on shared keys (e.g. `ENV`) and break one another if run on
/// multiple threads.
fn env_mutex() -> &'static Mutex<()> {
    static M: OnceLock<Mutex<()>> = OnceLock::new();
    M.get_or_init(|| Mutex::new(()))
}

fn with_var<F: FnOnce()>(key: &str, value: &str, f: F) {
    let _guard = env_mutex()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    // SAFETY: serialized under a mutex; keys used in tests are
    // not read by other threads concurrently.
    unsafe { env::set_var(key, value) };
    f();
    unsafe { env::remove_var(key) };
}

#[test]
fn try_get_env_parse_ok() {
    with_var("COMMONS_TRY_OK", "42", || {
        let v: Result<u32, _> = try_get_env("COMMONS_TRY_OK");
        assert_eq!(v.unwrap(), 42);
    });
}

#[test]
fn try_get_env_empty_value() {
    with_var("COMMONS_TRY_EMPTY", "", || {
        let v: Result<u32, _> = try_get_env("COMMONS_TRY_EMPTY");
        assert!(matches!(v, Err(EnvError::Empty(_))));
    });
}

#[test]
fn try_get_env_parse_error_with_context() {
    with_var("COMMONS_TRY_BAD", "not-a-number", || {
        let v: Result<u32, _> = try_get_env("COMMONS_TRY_BAD");
        match v {
            Err(EnvError::ParseError { var, value, .. }) => {
                assert_eq!(var, "COMMONS_TRY_BAD");
                assert_eq!(value, "not-a-number");
            }
            other => panic!("expected ParseError, got {other:?}"),
        }
    });
}

#[test]
fn get_string_set_empty_missing() {
    with_var("COMMONS_STR_SET", "hello", || {
        assert_eq!(get_string("COMMONS_STR_SET"), Some("hello".into()));
    });
    with_var("COMMONS_STR_EMPTY", "", || {
        assert_eq!(get_string("COMMONS_STR_EMPTY"), None);
    });
    assert_eq!(get_string("COMMONS_STR_MISSING_XYZ"), None);
}

#[test]
fn get_bool_truthy_and_falsy() {
    for (v, expected) in [
        ("1", true),
        ("true", true),
        ("True", true),
        ("yes", true),
        ("ON", true),
        ("0", false),
        ("false", false),
        ("no", false),
        ("", false),
    ] {
        let key = format!("COMMONS_BOOL_{v}_X");
        with_var(&key, v, || {
            assert_eq!(get_bool(&key), expected, "value={v}");
        });
    }
}

#[test]
fn get_list_splits_and_trims() {
    with_var("COMMONS_LIST_X", "a, b ,c,,d", || {
        let got = get_list("COMMONS_LIST_X", ",");
        assert_eq!(got, vec!["a", "b", "c", "d"]);
    });
}

#[test]
fn is_set_with_and_without_value() {
    with_var("COMMONS_IS_SET_VAR", "hi", || {
        assert!(is_set("COMMONS_IS_SET_VAR"));
    });
    with_var("COMMONS_IS_SET_EMPTY", "", || {
        assert!(!is_set("COMMONS_IS_SET_EMPTY"));
    });
}

#[test]
fn get_environment_reads_first_match() {
    with_var("ENV", "staging", || {
        assert_eq!(get_environment(), "staging");
    });
}

#[test]
fn is_production_is_development_is_test_helpers() {
    with_var("ENV", "production", || assert!(is_production()));
    with_var("ENV", "prod", || assert!(is_production()));
    with_var("ENV", "development", || assert!(is_development()));
    with_var("ENV", "dev", || assert!(is_development()));
    with_var("ENV", "test", || assert!(is_test()));
    with_var("ENV", "testing", || assert!(is_test()));
}

#[test]
fn require_env_success_path() {
    with_var("COMMONS_REQ_OK", "7", || {
        let v: u32 = require_env("COMMONS_REQ_OK");
        assert_eq!(v, 7);
    });
}

#[test]
#[should_panic(expected = "Required environment variable not set")]
fn require_env_panics_on_missing() {
    let _: String = require_env("COMMONS_REQ_DEFINITELY_MISSING_Q");
}

#[test]
#[should_panic(expected = "Cannot parse environment variable")]
fn require_env_panics_on_parse_error() {
    let _guard = env_mutex()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    // SAFETY: key is unique to this test and serialized via the
    // module mutex.
    unsafe { env::set_var("COMMONS_REQ_BAD", "nope") };
    let _: u32 = require_env("COMMONS_REQ_BAD");
}

#[test]
fn env_config_valid_when_required_var_set() {
    with_var("COMMONS_REQUIRED_VAR_X", "anything", || {
        let mut config = EnvConfig::new();
        let _ = config.require("COMMONS_REQUIRED_VAR_X");
        assert!(config.is_valid());
        assert!(config.validate().is_empty());
    });
}
