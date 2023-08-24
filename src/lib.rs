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
    println!("trying to restructure key: {k}");
    use serde_yaml::Value;
    if let Some((prefix, suffix)) = k.split_once('.') {
        if prefix.is_empty() || suffix.is_empty() {
            // Do nothing if we can't have both a prefix and a suffix
            return Ok(());
            println!("prefix and suffix are empty, done");
        }
        let val = m.remove(&k).unwrap();

        if !m.contains_key(prefix) {
            m.insert(Value::String(prefix.into()),
                     Value::Mapping(serde_yaml::Mapping::new()));

        }
        println!("{prefix}, {suffix} -> {:?}", val);
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
///
/// # Example
///
/// 
pub fn restructure_map(value: &mut serde_yaml::Value) -> Result<()> {
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

    Ok(())
}

/// Deserialize the string, then restructure it
pub fn restructure_from_str(s: &str) -> Result<serde_yaml::Value> {
    let mut value = serde_yaml::from_str(s)?;
    restructure_map(&mut value)?;

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
        let v1: Value = serde_yaml::from_str(s1).unwrap();
        let mut v2: Value = serde_yaml::from_str(s2).unwrap();
        restructure_map(&mut v2).unwrap();

        assert_eq!(v1, v2);
    }
}
