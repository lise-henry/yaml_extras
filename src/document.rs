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

/// Arguments passed to a `Documenter`.`format_key` closure.
pub struct KeyArgs<'k> {
    pub indent: &'k str,
    pub key: &'k str,
    pub description: Option<&'k str>,
    pub ty: &'k str,
    pub value: &'k str,
}

fn default_format_key(k: KeyArgs) -> String {
    let the_description = if let Some(s) = k.description {
        format!("# {s}\n")
    } else {
        "".to_owned()
    };
    let key = k.key;
    let ty = k.ty;
    let value = k.value;
    let indent = k.indent;
    format!("{indent}{the_description}{indent}{key}{ty}:{value}")
}


/// Contains the option for documenting YAML
pub struct Documenter<'d,> {
    indent: &'d str,
    description_field: &'d str,
    type_name: &'d dyn Fn(&ValueType) -> String,
    format_key: &'d dyn Fn(KeyArgs) -> String,
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
            format_key: &default_format_key,
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

    /// Change the way `Mappings` keys are displayed.
    ///
    /// # Example
    ///
    /// ```
    /// let yaml = serde_yaml::from_str::<serde_yaml::Value>(r#"foo: 42
    /// bar: true"#).unwrap();
    /// let actual = yaml_extras::Documenter::new()
    ///     // Quite useless way to display the info
    ///     .format_key(&|args| format!("{}!!! ", args.key.to_uppercase()))
    ///     .apply_value(&yaml, None)
    ///     .unwrap();
    ///
    /// assert_eq!(actual, "FOO!!! BAR!!! ");
    /// ```
    pub fn format_key(mut self, f: &'d dyn Fn(KeyArgs) -> String) -> Self {
        self.format_key = f;
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

    fn indent_str(&self, struct_path: &Vec<String>) -> String {
        let mut content = String::new();
        for _ in 0..struct_path.len() {
            content.push_str(self.indent);
        }
        content
    }


    fn document_val(&self, val: &Value, description: Option<&Value>, struct_path: &mut Vec<String>) -> error::Result<String> {
        let mut content = String::new();
        match val {
            Value::Mapping(ref m) => {
                if struct_path.len() > 0 {
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
                    let mut the_description: Option<&str> = None;
                    if let Some(inner) = desc_value {
                        match inner {
                            Value::String(s) => {
                                the_description = Some(s);
                            }
                            Value::Mapping(m) => {
                                // Try to see if there is a description field for this mapping
                                let desc = m.get(self.description_field)
                                    .and_then(|v| v.as_str());
                                if let Some(s) = desc {
                                    the_description = Some(s);
                                }
                            }
                            _ => {
                                
                            }
                        }
                    }
                    
                    
                    // Display the key name
                    let k = if key.is_string() {
                        key.as_str().unwrap().to_owned()
                    } else {
                        format!("{:?}", key)
                    };
                    struct_path.push(k.to_owned());
                    let v = self.document_val(value, desc_value, struct_path)?;
                    struct_path.pop();

                    let key_args = KeyArgs {indent: &self.indent_str(struct_path), key: &k, description: the_description, ty: &(*self.type_name)(&ty), value: &v};
                    content.push_str(&(*self.format_key)(key_args));
                }
            },
            Value::Sequence(ref s) => {
                content.push_str("\n");
                for v in s.iter() {
                    content.push_str(&self.indent_str(struct_path));
                    content.push_str("- ");
                    struct_path.push("-".to_owned());
                    content.push_str(&self.document_val(v, None, struct_path)?);
                    struct_path.pop();
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
        let mut struct_path = vec![];
        self.document_val(value, description, &mut struct_path)
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
    __description__: Description for foo.
    bar: Description for bar
"#;

        let yaml = r#"
foo:
    bar: 42
"#;

        let expected = r#"# Description for foo.
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
