// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::error;

use serde_yaml::Value;

#[derive(Debug, PartialEq)]
pub enum ValueType {
    Null,
    Bool,
    Number,
    String,
    List,
    Mapping,
    Tagged,
}

impl ValueType {
    pub fn to_str(v: &ValueType) -> String {
        match v {
            ValueType::Null | ValueType::Mapping | ValueType::Tagged => String::new(),
            __=> format!(" ({:?})", v)
        }
    }
}

const INDENT: &'static str = "    ";
const DESCRIPTION: &'static str = "__description__";

/// Contains the option for documenting YAML
pub struct Documenter<'d,> {
    indent: &'d str,
    description_field: &'d str,
    type_name: &'d dyn Fn(&ValueType) -> String,
}

impl<'d> Documenter<'d> {
    /// Creates a default documenter
    ///
    /// # Example
    ///
    /// ```
    /// let d = yaml_extras::Documenter::new();
    /// ```
    pub fn new() -> Self {
        Documenter {
            indent: INDENT,
            description_field: DESCRIPTION,
            type_name: &ValueType::to_str,
        }
    }

    /// Change the description field to describe a field that contains other field. Default: "__description__""
    ///
    /// E.g. if you have the following YAML structure:
    ///
    /// ```yaml
    /// foo:
    ///     bar: true
    ///     baz: false
    /// ```
    ///
    /// You can document it with the following YAML code:
    ///
    /// ```yaml
    /// foo:
    ///     __description__: This field is needed for foo because it contains nested fields
    ///     bar: No need for inned __description__ since bar contains only a value
    ///     baz: Same for baz
    /// ```
    ///
    /// By default you shouldn't need to change this, except if your YAML structure actually contains
    /// a field called `__description__`.
    pub fn description_field(mut self, field: &'d str) -> Self {
        self.description_field = field;
        self
    }

    /// Change the way to display types.
    ///
    /// The default function is a sensible one for english, but for other languages or if
    /// you want to tweak the display (e.g. not printing the type's name between parenthesis)
    /// you can change it.
    ///
    /// # Argument
    ///
    /// * f: reference to a (&ValueType) -> String closure or function. It it responsible for
    ///     returning the type name as string, but also the space before it (unless you
    ///    don't want to display the type). Typically you will want to match on `yaml_extras::document::ValueType`
    ///    and maybe call the `yaml_extras_document_ValueType::to_str` function, which is
    ///   the default.
    ///
    /// # Example
    /// ```
    /// let yaml = serde_yaml::from_str("foo: 42").unwrap();
    /// let mut d = yaml_extras::Documenter::new()
    ///     .type_name(&|t| format!(" (whatever)"));
    ///
    /// let mut actual = d.apply_value(&yaml, None).unwrap();
    /// assert_eq!(actual, "foo (whatever): 42\n");
    ///
    /// d = d.type_name(&|t| String::new());
    /// actual = d.apply_value(&yaml, None).unwrap();
    /// assert_eq!(actual, "foo: 42\n");
    /// ```
    pub fn type_name(mut self, f: &'d dyn Fn(&ValueType) -> String) -> Self {
        self.type_name = f;
        self
    }
    
    /// Change the indent. Default: 4 spaces.
    ///
    /// # Example
    ///
    /// ```
    /// let d = yaml_extras::Documenter::new()
    ///     .indent("\t");
    /// ```
    pub fn indent(mut self, indent: &'d str) -> Self {
        self.indent = indent;
        self
    }

    fn indent_str(&self, content: &mut String, n: u8) {
        for _ in 0..n {
            content.push_str(self.indent);
        }
    }


    fn document_val(&self, val: &Value, description: Option<&Value>, indent_level: u8) -> error::Result<String> {
        let mut content = String::new();
        match val {
            Value::Mapping(ref m) => {
                if indent_level > 0 {
                    content.push_str("\n");
                }
                for (key, value) in m.iter() {
                    let ty = match value {
                        Value::Null => ValueType::Null,
                        Value::Bool(_) => ValueType::Bool,
                        Value::Number(_) => ValueType::Number,
                        Value::String(_) => ValueType::String,
                        Value::Sequence(_) => ValueType::List,
                        Value::Mapping(_) => ValueType::Mapping,
                        Value::Tagged(_) => ValueType::Tagged,
                    };
                    // Try displaying the description, if it exists
                    let desc_value = description.and_then(|d| d.as_mapping())
                        .and_then(|m| m.get(key));
                    if let Some(inner) = desc_value {
                        match inner {
                            Value::String(s) => {
                                // Found a description, displays it
                                self.indent_str(&mut content, indent_level);
                                content.push_str("# ");
                                content.push_str(s);
                                content.push_str("\n");
                            }
                            Value::Mapping(m) => {
                                // Try to see if there is a description field for this mapping
                                let desc = m.get(self.description_field)
                                    .and_then(|v| v.as_str());
                                if let Some(s) = desc {
                                    self.indent_str(&mut content, indent_level);
                                    content.push_str("# ");
                                    content.push_str(s);
                                    content.push_str("\n");
                                }
                            }
                            _ => {
                                
                            }
                        }
                    }
                    
                    
                    // Display the key name
                    self.indent_str(&mut content, indent_level);
                    let k = if key.is_string() {
                        key.as_str().unwrap().to_owned()
                    } else {
                        format!("{:?}", key)
                    };
                    content.push_str(&format!("{k}{t}:", t = (*self.type_name)(&ty)));
                    content.push_str(&self.document_val(value, desc_value, indent_level + 1)?);
                }
            },
            Value::Sequence(ref s) => {
                content.push_str("\n");
                for v in s.iter() {
                    self.indent_str(&mut content, indent_level);
                    content.push_str("- ");
                    content.push_str(&self.document_val(v, None, indent_level + 1)?);
                }
            }
            _ => {
                content.push_str(" ");
                content.push_str(&serde_yaml::to_string(val)?);
            },
        }
        Ok(content)    
    }


    /// Uses a YAML representation from a default struct to document an API or options
    /// in a YAML-looking way
    ///
    /// The idea is to first generate a yaml representation with YourStruct::default()
    /// to get a mostly automated API description.
    ///
    /// # Arguments
    ///
    /// * `value`: should correspond to a `serde_yaml` Value with the default values of your
    ///   structure
    /// * `description`: an optional `serde_yaml` value mirroring the `value` but with descriptions for
    ///    fields you want to document. Use `__description__` inside a `Mapping` to document the
    ///   upper-level field.
    ///
    /// # Example
    ///
    /// ```
    ///         let desc_yaml = r#"
    /// foo:
    ///     __description__: Description for foo
    ///     bar: Description for bar
    /// "#;
    /// 
    ///         let yaml = r#"
    /// foo:
    ///     bar: 42
    /// "#;
    /// 
    ///         let expected = "# Description for foo
    /// foo:\n    # Description for bar
    ///     bar (Number): 42
    /// ";
    ///         let value: serde_yaml::Value = serde_yaml::from_str(&yaml).unwrap();
    ///         let desc: serde_yaml::Value = serde_yaml::from_str(&desc_yaml).unwrap();
    ///         let s = yaml_extras::Documenter::new()
    ///             .apply_value(&value, Some(&desc))
    ///             .unwrap();
    ///         assert_eq!(s, expected);
    /// ```
    pub fn apply_value(&self, value: &Value, description: Option<&Value>) -> error::Result<String> {
        self.document_val(value, description, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::{assert_eq, assert_ne};
    use serde_yaml::Value;

    #[test]
    fn document_simple() {
        let desc_yaml = r#"
foo:
    __description__: Description for foo
    bar: Description for bar
"#;

        let yaml = r#"
foo:
    bar: 42
"#;

        let expected = r#"# Description for foo
foo:
    # Description for bar
    bar (Number): 42
"#;
        let value: Value = serde_yaml::from_str(&yaml).unwrap();
        let desc: Value = serde_yaml::from_str(&desc_yaml).unwrap();
        let s = Documenter::new()
            .apply_value(&value, Some(&desc)).unwrap();
        assert_eq!(s, expected);
    }

    #[test]
    fn indent() {
        let desc_yaml = r#"
foo:
    __description__: Description for foo
    bar: Description for bar
"#;

        let yaml = r#"
foo:
    bar: 42
"#;

        let expected = r#"# Description for foo
foo:
....# Description for bar
....bar (Number): 42
"#;
        let value: Value = serde_yaml::from_str(&yaml).unwrap();
        let desc: Value = serde_yaml::from_str(&desc_yaml).unwrap();
        let s = Documenter::new()
            .indent("....")
            .apply_value(&value, Some(&desc)).unwrap();
        assert_eq!(s, expected);
    }

    #[test]
    fn description() {
        let desc_yaml = r#"
foo:
    ___description___: Description for foo
    __description__: Description for __description__
"#;

        let yaml = r#"
foo:
    __description__: 42
"#;

        let expected = r#"# Description for foo
foo:
....# Description for __description__
....__description__ (Number): 42
"#;
        let value: Value = serde_yaml::from_str(&yaml).unwrap();
        let desc: Value = serde_yaml::from_str(&desc_yaml).unwrap();
        let s = Documenter::new()
            .indent("....")
            .description_field("___description___")
            .apply_value(&value, Some(&desc)).unwrap();
        assert_eq!(s, expected);
    }
}
