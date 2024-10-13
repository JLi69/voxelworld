#[cfg(test)]
use super::*;

#[test]
fn test_impfile1() {
    let entries = parse_file("test_impfiles/test1.impfile");
    assert_eq!(entries.len(), 1);
    let e = &entries[0];
    assert_eq!(e.get_name(), "entry1");
    assert_eq!(e.variables.len(), 2);
    assert!(e.variables.contains_key("var1"));
    assert!(e.variables.contains_key("var2"));
    assert_eq!(e.get_var("var1"), "hello");
    assert_eq!(e.get_var("var2"), "good bye");
}

#[test]
fn test_impfile2() {
    let entries = parse_file("test_impfiles/test2.impfile");
    assert_eq!(entries.len(), 3);
    let expected = vec![
        Entry::from_vec(
            "foo",
            vec![
                ("a".to_string(), "bar".to_string()),
                ("b".to_string(), "baz".to_string()),
                ("c".to_string(), "buzz".to_string()),
            ],
        ),
        Entry::from_vec(
            "foo2",
            vec![
                ("var1".to_string(), "fizz".to_string()),
                ("var2".to_string(), "buzz".to_string()),
            ],
        ),
        Entry::new("foo3"),
    ];

    for (i, e) in expected.iter().enumerate() {
        assert_eq!(e.get_name(), entries[i].get_name());
        for (name, value) in &entries[i].variables {
            assert!(e.variables.contains_key(name));
            assert_eq!(e.get_var(name), *value);
            assert_eq!(e.get_var(name), entries[i].get_var(name));
        }
    }
}

#[test]
fn test_impfile3() {
    let entries = parse_file("test_impfiles/test3.impfile");
    assert!(entries.is_empty());
}

#[test]
fn test_impfile4() {
    let entries = parse_file("test_impfiles/test4.impfile");
    assert!(entries.is_empty());
}

#[test]
fn test_entry_to_string() {
    let mut entry = Entry::new("test");
    entry.add_string("foo", "bar");
    entry.add_string("fizz", "buzz");

    let entry_str = strip_whitespace(&entry.to_impfile_string());
    let mut entry_str_chars = entry_str.chars().peekable();

    let entry2 = parse_entry(&mut entry_str_chars);
    assert!(entry2.is_ok());

    if let Ok(e) = entry2 {
        assert_eq!(e.get_name(), entry.get_name());
        for (name, value) in &entry.variables {
            assert!(entry.variables.contains_key(name));
            assert_eq!(e.get_var(&name), *value);
            assert_eq!(e.get_var(&name), entry.get_var(&name));
        }
    }
}

#[test]
fn test_add_variables() {
    let mut entry = Entry::new("test");
    entry.add_bool("test_bool1", true);
    entry.add_bool("test_bool2", false);
    entry.add_float("test_float", 1.23);
    assert_eq!(entry.get_var("test_bool1"), "true");
    assert_eq!(entry.get_var("test_bool2"), "false");
    assert_eq!(entry.get_var("test_float"), "1.23");
}
