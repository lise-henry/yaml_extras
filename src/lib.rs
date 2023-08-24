// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Misc yaml-related utility functions.
//!
//! # Restructure
//!
//! If you use YAML for a configuration file, you might want to allow to use
//! both things like:
//!
//! ```yaml
//! compiler:
//!     command: cargo build
//! ```
//!
//! and:
//!
//! ```yaml
//! compiler.command: cargo build
//! ```
//!
//! (Or not. I know *I* needed that. Whatever.)
//!
//! The functions `restructure_map` and `restructure_from_str` allow just that,
//! converting dotted keys to inner fiels:
//!
//! ```
//!         let s1 = r#"
//! compiler:
//!     command: cargo build
//! "#;
//! 
//!         let s2 = r#"
//! compiler.command: cargo build
//! "#;
//!         let v1: serde_yaml::Value = serde_yaml::from_str(s1).unwrap();
//!         let v2 = yaml_extras::restructure_from_str(&s2, true).unwrap();
//!         assert_eq!(v1, v2);
//! ```

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("impossible to restructure YAML map: {0}")]
    Restructure(String),
    #[error("YAML error")]
    Yaml(#[from] serde_yaml::Error)
}

pub type Result<T> = std::result::Result<T, Error>;


/// Restructure a key inside a mapping so that if it's dotted it will be inserted
/// to submap.
fn restructure_key(m: &mut serde_yaml::Mapping, k: &str) -> Result<()> {
    use serde_yaml::Value;
    if let Some((prefix, suffix)) = k.split_once('.') {
        if prefix.is_empty() || suffix.is_empty() {
            // Do nothing if we can't have both a prefix and a suffix
            return Ok(());
        }
        let val = m.remove(&k).unwrap();

        if !m.contains_key(prefix) {
            m.insert(Value::String(prefix.into()),
                     Value::Mapping(serde_yaml::Mapping::new()));

        }
        let inner = m.get_mut(prefix)
            .unwrap()
            .as_mapping_mut()
            .ok_or(Error::Restructure(format!("could not insert key {k}: {prefix} is not a mapping")))?;
        inner.insert(Value::String(suffix.into()),
                     val);
        // Check the inner map and the suffix to see if it still contains dots
        restructure_key(inner, suffix)?;
    }
    Ok(())
}


/// Restructure a YAML map so that keys containing dots are transformed into appropriate
/// fields of sub-maps.
///
/// E.g. `foo.bar.baz: true" will convert to
///
/// ```yaml
/// foo:
///     bar:
///         baz: true
/// ```
///
/// # Arguments
///
/// * value: a value obtained from serde_yaml; should be a Mapping.
/// * recursive: set to `true` to apply recursively to all values inside the map,
///   with `false` it will only be applied for keys at top-level
///
/// # Example
///
/// ```
///         let s1 = r#"
/// foo:
///     bar:
///         baz: 42
/// "#;
/// 
///         let s2 = r#"
/// foo.bar.baz: 42
/// "#;
///         let mut v1: serde_yaml::Value = serde_yaml::from_str(s1).unwrap();
///         let mut v2: serde_yaml::Value = serde_yaml::from_str(s2).unwrap();
///         yaml_extras::restructure_map(&mut v2, false).unwrap();
///         assert_eq!(v1, v2);
/// 
/// ```
///
/// 
pub fn restructure_map(value: &mut serde_yaml::Value, recursive: bool) -> Result<()> {
    use serde_yaml::Value;
    let m = value.as_mapping_mut()
        .ok_or(Error::Restructure("not a mapping".into()))?;
    let dotted_keys: Vec<String> = m.keys()
        .filter(|v| {
            let mut res = false;
            if let Value::String(ref s) = v  {
                if let Some(c) = s.find('.') {
                    if c > 0 && c < s.len() - 1 {
                        res = true;
                    }
                }
            }
            res
        })
        .map(|v| v.as_str()
             .unwrap()
             .to_owned())
        .collect();
    for k in dotted_keys {
        restructure_key(m, &k)?;
    }

    if recursive {
        let map_keys: Vec<String> = m.iter()
            .filter(|(k, v)| k.is_string() && v.is_mapping())
            .map(|(k, _)| k.as_str().unwrap().to_owned())
            .collect();
        for k in map_keys {
            restructure_map (m.get_mut(&k)
                             .unwrap(),
                             true)?;
            
        }
    }

    Ok(())
}

/// Deserialize the string, then restructure it
pub fn restructure_from_str(s: &str, recursive: bool) -> Result<serde_yaml::Value> {
    let mut value = serde_yaml::from_str(s)?;
    restructure_map(&mut value, recursive)?;

    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::{assert_eq, assert_ne};
    use serde_yaml::Value;

    #[test]
    fn test_simple() {
        let s1 = r#"
foo:
    bar:
        baz: true
"#;

        let s2 = r#"
foo.bar.baz: true
"#;
        let mut v1: Value = serde_yaml::from_str(s1).unwrap();
        let mut v2: Value = serde_yaml::from_str(s2).unwrap();
        restructure_map(&mut v2, false).unwrap();
        assert_eq!(v1, v2);

        v2 = serde_yaml::from_str(s2).unwrap();
        restructure_map(&mut v2, true).unwrap();
        assert_eq!(v1, v2);

        restructure_map(&mut v1, true).unwrap();
        assert_eq!(v1, v2);
    }

    #[test]
    fn test_recursion1() {
        let s1 = r#"
foo:
    bar:
        baz: true
"#;

        let s2 = r#"
foo:
    bar.baz: true
"#;
        let v1: Value = serde_yaml::from_str(s1).unwrap();
        let mut v2: Value = serde_yaml::from_str(s2).unwrap();
        restructure_map(&mut v2, false).unwrap();
        assert_ne!(v1, v2);

        v2 = serde_yaml::from_str(s2).unwrap();
        restructure_map(&mut v2, true).unwrap();
        assert_eq!(v1, v2);
    }

    #[test]
    fn test_error() {
        let s1 = r#"
foo:
    bar: 42
foo.bar.baz: true
"#;

        let mut v1: Value = serde_yaml::from_str(s1).unwrap();
        let res = restructure_map(&mut v1, false);
        assert!(res.is_err());
    }

    #[test]
    fn test_from_str() {
        let s1 = r#"
foo:
    bar:
        baz: true
"#;

        let s2 = r#"
foo.bar.baz: true
"#;
        let v1: Value = serde_yaml::from_str(s1).unwrap();
        let v2: Value = restructure_from_str(&s2, true).unwrap();
        assert_eq!(v1, v2);
    }
}
