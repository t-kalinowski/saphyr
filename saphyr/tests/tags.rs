use saphyr::{LoadableYamlNode, Scalar, Yaml};

#[test]
fn f() {
    use saphyr::Tag;
    let parse = |s| Yaml::load_from_str(s).unwrap().into_iter().next().unwrap();

    let custom_tag = &Tag {
        handle: "!".into(),
        suffix: "custom".into(),
    };
    let parsed = parse("!custom 3");
    assert!(
        matches!(parsed, Yaml::Tagged(tag, node) if tag.as_ref() == custom_tag && node.is_integer())
    );
    assert!(
        matches!(parse("!custom 'foo'"), Yaml::Tagged(tag, node) if tag.as_ref() == custom_tag && node.is_string())
    );
    assert!(
        matches!(parse("!custom foo"), Yaml::Tagged(tag, node) if tag.as_ref() == custom_tag && node.is_string())
    );
    assert!(
        matches!(parse("!custom ~"), Yaml::Tagged(tag, node) if tag.as_ref() == custom_tag && node.is_null())
    );
    assert!(
        matches!(parse("!custom '3'"), Yaml::Tagged(tag, node) if tag.as_ref() == custom_tag && node.is_string())
    );
}

#[test]
fn on_scalar() {
    let s = "
- !degree 45
- foo
";
    let docs = Yaml::load_from_str(s).unwrap();
    let doc = &docs[0];

    assert!(doc.is_sequence());
    let items = doc.as_sequence().unwrap();

    let foo = &items[1];
    assert!(foo.as_str().is_some_and(|s| s == "foo"));

    let degrees = &items[0];
    let Yaml::Tagged(tag, degree) = degrees else {
        panic!("Not a Tagged")
    };
    let tag = tag.as_ref();
    assert!(tag.handle == "!");
    assert!(tag.suffix == "degree");
    assert!(degree.as_integer().is_some_and(|x| x == 45));
}

#[test]
fn on_collection() {
    let s = "
- !degrees [45, 30]
- foo
";
    let docs = Yaml::load_from_str(s).unwrap();
    let doc = &docs[0];

    assert!(doc.is_sequence());
    let items = doc.as_sequence().unwrap();

    let foo = &items[1];
    assert!(foo.as_str().is_some_and(|s| s == "foo"));

    let degrees = &items[0];
    let Yaml::Tagged(tag, degrees) = degrees else {
        panic!("Not a Tagged")
    };
    let tag = tag.as_ref();
    assert!(tag.handle == "!");
    assert!(tag.suffix == "degrees");

    let arr = degrees.as_sequence().unwrap();
    assert!(arr[0].as_integer().is_some_and(|x| x == 45));
    assert!(arr[1].as_integer().is_some_and(|x| x == 30));
}

#[test]
fn on_collection_and_item() {
    let s = "
- !degrees [45, !inner_tag 30, 'untagged_string']
- foo
";
    let docs = Yaml::load_from_str(s).unwrap();
    let doc = &docs[0];

    assert!(doc.is_sequence());
    let items = doc.as_sequence().unwrap();

    let foo = &items[1];
    assert!(foo.as_str().is_some_and(|s| s == "foo"));

    let collection = &items[0];
    let Yaml::Tagged(tag, degrees) = collection else {
        panic!("Not a Tagged")
    };
    let tag = tag.as_ref();
    assert!(tag.handle == "!");
    assert!(tag.suffix == "degrees");

    let arr = degrees.as_sequence().unwrap();
    assert!(arr[0].as_integer().is_some_and(|x| x == 45));
    assert!(arr[2].as_str().is_some_and(|s| s == "untagged_string"));

    let Yaml::Tagged(ref inner_tag, ref degree) = arr[1] else {
        panic!("Not a tagged")
    };
    assert!(inner_tag.handle == "!");
    assert!(inner_tag.suffix == "inner_tag");
    assert!(degree.as_integer().is_some_and(|x| x == 30));
}

#[test]
fn core_schema_collection_tag() {
    let s = "
- !!seq []
- !!str 12
- !!int 12
";
    let docs = Yaml::load_from_str(s).unwrap();
    let doc = &docs[0];

    assert!(doc.is_sequence());
    let items = doc.as_sequence().unwrap();
    assert!(items[0].as_sequence().is_some_and(Vec::is_empty));
    assert!(items[1].as_str().is_some_and(|s| s == "12"));
    assert!(items[2].as_integer().is_some_and(|i| i == 12));
}

#[test]
fn verbatim_core_tags_match_primary_handle() {
    let docs = Yaml::load_from_str(
        "
- !!str foo
- !<tag:yaml.org,2002:str> foo
- !<tag:yaml.org,2002:int> 2
",
    )
    .unwrap();
    let doc = &docs[0];

    let seq = doc.as_sequence().expect("document sequence");
    assert!(matches!(seq[0], Yaml::Value(Scalar::String(ref s)) if s == "foo"));
    assert!(matches!(seq[1], Yaml::Value(Scalar::String(ref s)) if s == "foo"));
    assert!(matches!(seq[2], Yaml::Value(Scalar::Integer(i)) if i == 2));
}

#[test]
fn verbatim_core_prefix_unknown_suffix_is_not_split() {
    let docs = Yaml::load_from_str("- !<tag:yaml.org,2002:foo> 1").unwrap();
    let seq = docs[0].as_sequence().expect("document sequence");
    let Yaml::Tagged(tag, node) = &seq[0] else {
        panic!("expected Tagged");
    };
    assert!(tag.handle.is_empty());
    assert_eq!(tag.suffix, "tag:yaml.org,2002:foo");
    assert!(matches!(**node, Yaml::Value(Scalar::Integer(1))));
}

#[test]
fn unknown_core_tag_on_mapping_is_preserved() {
    let docs = Yaml::load_from_str("!!foo {bar: 1}").unwrap();
    let Yaml::Tagged(tag, node) = &docs[0] else {
        panic!("expected Tagged");
    };
    assert_eq!(tag.handle, "tag:yaml.org,2002:");
    assert_eq!(tag.suffix, "foo");

    let map = node.as_mapping().expect("inner mapping");
    assert_eq!(map.len(), 1);
    let (k, v) = map.iter().next().expect("entry");
    assert!(k.as_str().is_some_and(|s| s == "bar"));
    assert!(v.as_integer().is_some_and(|i| i == 1));
}

#[test]
fn core_set_tag_is_preserved_as_tagged_mapping() {
    let docs = Yaml::load_from_str("!!set {bar, baz}").unwrap();
    let Yaml::Tagged(tag, node) = &docs[0] else {
        panic!("expected Tagged");
    };
    assert_eq!(tag.handle, "tag:yaml.org,2002:");
    assert_eq!(tag.suffix, "set");

    let map = node.as_mapping().expect("inner mapping");
    assert_eq!(map.len(), 2);
    assert!(map.contains_key(&Yaml::Value(Scalar::String("bar".into()))));
    assert!(map.contains_key(&Yaml::Value(Scalar::String("baz".into()))));
}
