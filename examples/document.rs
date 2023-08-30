use serde_derive::Serialize;
use std::default::Default;

#[derive(Serialize, Default)]
struct UserId {
    first_name: String,
    last_name: String,
    public_id: String,
    internal_id: u32,
}

#[derive(Serialize)]
struct Preferences {
    screen_size: [u32; 2],
    allow_cookies: bool,
    two_fa: bool
}

impl Default for Preferences {
    fn default() -> Self {
        Preferences {
            screen_size: [1024, 768],
            allow_cookies: false,
            two_fa: true
        }
    }
}

#[derive(Serialize, Default)]
struct User {
    id: UserId,
    preferences: Preferences,
}


fn main() {
    use yaml_extras::document::ValueType;


    let desc_yaml = r#"
id:
    __description__: Info about the users
    public_id: Must oncly contain aphanumeric characters
    internal_id: Must never be changed
preferences:
    __description__: Various user settings
    screen_size: Width and height, in pixels, obviously
"#;

    let value = serde_yaml::to_value(&User::default()).unwrap();
    let desc: serde_yaml::Value = serde_yaml::from_str(&desc_yaml).unwrap();
    let d2 = yaml_extras::Documenter::new();
    let d = yaml_extras::Documenter::new()
        .format_key(&|k| {
            format!("{}{}:{}\n", k.indent, k.key, k.value)
        })
        .type_name(&|t| match t {
            ValueType::Mapping | ValueType::Tagged => format!(""),
            _ => ValueType::to_str(t)
        });
    let s = d2.apply_value(&value, Some(&desc)).unwrap();

    println!("{s}");
}
