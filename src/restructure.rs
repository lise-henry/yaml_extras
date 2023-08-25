// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::error::Result;
use crate::error::Error;

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
///         let v1: serde_yaml::Value = serde_yaml::from_str(s1)?;
///         let v2 = yaml_extras::Restructurer::new()
///             .from_str(s2)?;
///         assert_eq!(v1, v2);
/// # Ok::<(), yaml_extras::Error>(())
/// 
/// ```
///
/// This struct mainly stores the options so they are easier to set/pass than tons of
/// arguments to a single function
pub struct Restructurer<'r> {
    recursive: bool,
    ignore: Vec<&'r str>,
}

impl<'r> Restructurer<'r> {
    /// Creates a new Restructurer with default values
    pub fn new() -> Self {
        Restructurer {
            recursive: true,
            ignore: vec![],
        }
    }

    /// Set to `true` to apply restructuration recursively to all values inside the map,
    ///  with `false` it will only be applied for keys at top-level (default is `false`)
    ///
    /// # Example
    ///
    /// ```
    /// let restructurer = yaml_extras::Restructurer::new()
    ///     .recursive(false);
    /// ```
    pub fn recursive(mut self, recursive: bool) -> Self {
        self.recursive = recursive;
        self
    }

    /// Add some (dotted) values that should be ignored in case you actually use dotted keys
    ///
    /// I mean if you use dotted keys you probably woudn't want to restructure your yaml representation anyway
    /// except in stupid cases where you still need to restructure in some cases but not others
    /// because of silly retrocompatbility issues. Not saying that this option is present
    /// here because I hadn't thought too much about the keys of my options configuration file,
    /// I mean it's just in case you need it, me I woudn't need that no ahhaha :sob:
    ///
    /// # Example
    ///
    /// ```
    /// let e = r#"
    /// some.key: 42
    /// foo:
    ///     another.key:
    ///         bar: true
    /// "#;
    ///
    /// let s = r#"
    /// some.key: 42
    /// foo.another.key.bar: true
    /// "#;
    ///
    /// let expected: serde_yaml::Value = serde_yaml::from_str(e)?;
    /// let actual = yaml_extras::Restructurer::new()
    ///     .ignore(vec!["some.key", "another.key"])
    ///     .from_str(s)?;
    /// assert_eq!(actual, expected);
    /// # Ok::<(), yaml_extras::Error>(())
    /// ```
    pub fn ignore (mut self, ignore: Vec<&'r str>) -> Self {
        self.ignore = ignore;
        self
    }
    /// # Example
    ///
    /// ```
    /// let s1 = r#"
    /// foo:
    ///     bar:
    ///         baz: 42
    /// "#;
    /// 
    /// let s2 = r#"
    /// foo.bar.baz: 42
    /// "#;
    ///
    /// let mut v1: serde_yaml::Value = serde_yaml::from_str(s1)?;
    /// let mut v2: serde_yaml::Value = serde_yaml::from_str(s2)?;
    /// yaml_extras::Restructurer::new()
    ///             .apply_to_value(&mut v2)?;
    /// assert_eq!(v1, v2);
    /// # Ok::<(), yaml_extras::Error>(())
    /// ```
    pub fn apply_to_value(self: &Self, value: &mut serde_yaml::Value) -> Result<()> {
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
            self.restructure_key(m, &k)?;
        }
        
        if self.recursive {
            let map_keys: Vec<String> = m.iter()
                .filter(|(k, v)| k.is_string() && v.is_mapping())
                .map(|(k, _)| k.as_str().unwrap().to_owned())
                .collect();
            for k in map_keys {
                self.apply_to_value(m.get_mut(&k)
                                    .unwrap())?;
                
            }
        }
        
        Ok(())
    }

    /// Deserialize the string to YAML representation, then restructure it and retuns a
    /// `serde_yaml::Value`
    ///
    /// # Example
    ///
    /// ```
    /// let yaml = "nested.key: 42";
    ///
    /// let value = yaml_extras::Restructurer::new()
    ///     .from_str(yaml)?;
    /// 
    /// # Ok::<(), yaml_extras::Error>(())
    /// ```
    ///
    /// Now `value` will correspond to (the `serde_yaml` internal representation of)
    ///
    /// ```yaml
    /// nested:
    ///     key: 42
    /// ```
    pub fn from_str(self: &Self, s: &str) -> Result<serde_yaml::Value> {
        let mut value = serde_yaml::from_str(s)?;
        self.apply_to_value(&mut value)?;
        
        Ok(value)
    }


    
    /// Restructure a key inside a mapping so that if it's dotted it will be inserted
    /// to submap.
    fn restructure_key(self: &Self, m: &mut serde_yaml::Mapping, k: &str) -> Result<()> {
        use serde_yaml::Value;

        if let Some((mut prefix, mut suffix)) = k.split_once('.') {
            // Check if the key is in the ignore list
            for i in &self.ignore {
                if k.starts_with(i) {
                    // k is in ignore list, revamp prefix and suffix
                    if let Some((p, s)) = k.split_once(&format!("{i}.")) {
                        prefix = i;
                        suffix = s;
                        break;
                    } else {
                        // Nothing besides, returnning
                        return Ok(());
                    }
                }
            }

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
            self.restructure_key(inner, suffix)?;
        }
    
    Ok(())
}
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
        let mut r = Restructurer::new()
            .recursive(false);

        r.apply_to_value(&mut v2).unwrap();
        assert_eq!(v1, v2);

        v2 = serde_yaml::from_str(s2).unwrap();
        r = r.recursive(true);
        r.apply_to_value(&mut v2).unwrap();
        assert_eq!(v1, v2);

        r.apply_to_value(&mut v1).unwrap();
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
        Restructurer::new()
            .recursive(false)
            .apply_to_value(&mut v2).unwrap();
        assert_ne!(v1, v2);

        v2 = serde_yaml::from_str(s2).unwrap();
        Restructurer::new()
            .apply_to_value(&mut v2).unwrap();
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
        let res = Restructurer::new()
            .apply_to_value(&mut v1);
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
        let v2: Value = Restructurer::new()
            .from_str(&s2)
            .unwrap();
        assert_eq!(v1, v2);
    }

    #[test]
    fn ignore() {
        let s1 = r#"
foo:
    ignored.key:
        baz: true
"#;

        let s2 = r#"
foo.ignored.key.baz: true
"#;
        let v1: Value = serde_yaml::from_str(s1).unwrap();
        let v2: Value = Restructurer::new()
            .ignore(vec!["ignored.key"])
            .from_str(&s2)
            .unwrap();
        assert_eq!(v1, v2);
    }
}

