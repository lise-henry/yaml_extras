fn main() {

    let desc_yaml = r#"
animal:
    __description__: An animal
    cat: The best animal
    ant: An insect
"#;

    let yaml = r#"
foo: 42
bar: true
animal:
    cat:
        legs: 4
        say: ["Nyaaah", "Meow", "rrrrr"]
    ant:
        legs: 6
"#;

    let value: serde_yaml::Value = serde_yaml::from_str(&yaml).unwrap();
    let desc: serde_yaml::Value = serde_yaml::from_str(&desc_yaml).unwrap();
    let s = yaml_extras::document(&value, Some(&desc)).unwrap();

    println!("{s}");
}
