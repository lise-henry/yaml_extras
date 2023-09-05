fn main() {
    let yaml1 = r#"
foo:
    test: true
bar: 42"#;

    let yaml2 = r#"
foo:
    other_test: false
baz: 32"#;

    let mut v1: serde_yaml::Value = serde_yaml::from_str(yaml1).unwrap();
    let v2: serde_yaml::Value = serde_yaml::from_str(yaml2).unwrap();
    yaml_extras::merge(&mut v1, &v2).unwrap();

    println!("{:?}", v1);
}
