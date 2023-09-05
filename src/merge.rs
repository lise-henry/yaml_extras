// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::error::{Result, Error};

use serde_yaml::Value;


/// Merge two YAML representations into another
///
pub fn merge(value: &mut Value, other: &Value) -> Result<()> {
    if let (Some(v), Some(o))  = (value.as_mapping_mut(), other.as_mapping()) {
        for (o_key, o_val) in o.iter() {
            if !o_val.is_mapping() {
                v.insert(o_key.clone(), o_val.clone());
            } else {
                /// If the contained hashmap  is already present, merge the hashmap
                if v.contains_key(o_key) {
                    merge(v.get_mut(o_key).unwrap(), o_val)?;
                } else {
                    v.insert(o_key.clone(), o_val.clone());
                }
            }
        }
        return Ok(())
    }
    return Err(Error::Merge(format!("both arguments need to be mapping, found {:?} and {:?}", value, other))); 
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::{assert_eq, assert_ne};

    #[test]
    fn merge_simple() {
        let y1 = r#"
foo: 42"#;
        let y2 = r#"
bar: true"#;

        let expected = r#"
foo: 42
bar: true"#;

        let mut actual: Value = serde_yaml::from_str(y1).unwrap();
        let v2: Value = serde_yaml::from_str(y2).unwrap();
        let expected: Value = serde_yaml::from_str(expected).unwrap();

        merge(&mut actual, &v2).unwrap();

        assert_eq!(actual, expected);
    }
}
