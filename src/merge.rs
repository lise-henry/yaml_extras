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
