#[cfg(test)]
use super::*;

#[test]
fn test_bin_data1() {
    //Test data
    let mut table = DataTable::new();
    table.add_int("a", 1);

    //Convert data into bytes
    let bytes = get_table_list_bytes("test", &vec![table]);
    assert!(!bytes.is_empty());

    //Parse the data
    let mut stream = ByteStream::new(bytes);
    let parsed = parse_binary_data(&mut stream);

    //Make sure it's not empty
    assert_eq!(parsed.len(), 1);
    assert!(parsed.contains_key("test"));

    //Check if the data is correct
    if let Some(tables) = parsed.get("test") {
        assert_eq!(tables.len(), 1);
        assert_eq!(tables[0].get_int("a"), Some(1));
    }
}

#[test]
fn test_bin_data2() {
    //Test data
    let a = 0xff;
    let b = 0.5;
    let c = vec3(1.0, 1.0, 1.0);
    let d = "hello world";
    let mut table = DataTable::new();
    table.add_int("a", a);
    table.add_float("b", b);
    table.add_vec3("c", c);
    table.add_str("d", d);

    //Convert data into bytes
    let bytes = get_table_list_bytes("test", &vec![table]);
    assert!(!bytes.is_empty());

    //Parse the data
    let mut stream = ByteStream::new(bytes);
    let parsed = parse_binary_data(&mut stream);

    //Make sure it's not empty
    assert_eq!(parsed.len(), 1);
    assert!(parsed.contains_key("test"));

    //Check if the data is correct
    if let Some(tables) = parsed.get("test") {
        assert_eq!(tables.len(), 1);
        assert_eq!(tables[0].get_int("a"), Some(a));
        assert_eq!(tables[0].get_float("b"), Some(b));
        assert_eq!(tables[0].get_vec3("c"), Some(c));
        assert_eq!(tables[0].get_str("d"), Some(d.to_string()));
    }
}

#[test]
fn test_bin_data3() {
    //Test data
    let mut tables = vec![];
    for i in 0..10 {
        let mut table = DataTable::new();
        table.add_int("foo", i);
        table.add_str("bar", "blahblah");
        tables.push(table);
    }

    //Convert data into bytes
    let bytes = get_table_list_bytes("test", &tables);
    assert!(!bytes.is_empty());

    //Parse the data
    let mut stream = ByteStream::new(bytes);
    let parsed = parse_binary_data(&mut stream);

    //Make sure it's not empty
    assert_eq!(parsed.len(), 1);
    assert!(parsed.contains_key("test"));

    //Check if the data is correct
    if let Some(tables) = parsed.get("test") {
        assert_eq!(tables.len(), 10);
        for (i, table) in tables.iter().enumerate() {
            assert_eq!(table.get_int("foo"), Some(i as i64));
            assert_eq!(table.get_str("bar"), Some("blahblah".to_string()));
        }
    }
}

#[test]
fn test_bin_data4() {
    //Test data
    let mut tables1 = vec![];
    for i in 0..10 {
        let mut table = DataTable::new();
        table.add_int("foo", i);
        table.add_str("bar", "blahblah");
        tables1.push(table);
    }

    let mut tables2 = vec![];
    for i in 0..5 {
        let mut table = DataTable::new();
        let i_f32 = i as f32;
        table.add_float("fizz", i_f32);
        table.add_vec3("buzz", vec3(i_f32, i_f32, i_f32));
        tables2.push(table);
    }

    //Convert data into bytes
    let mut bytes = vec![];
    bytes.extend(get_table_list_bytes("foo", &tables1));
    bytes.extend(get_table_list_bytes("bar", &tables2));

    //Parse the data
    let mut stream = ByteStream::new(bytes);
    let parsed = parse_binary_data(&mut stream);

    //Make sure it's not empty
    assert_eq!(parsed.len(), 2);
    assert!(parsed.contains_key("foo"));
    assert!(parsed.contains_key("bar"));

    //Check if the data is correct
    if let Some(tables) = parsed.get("foo") {
        assert_eq!(tables.len(), 10);
        for (i, table) in tables.iter().enumerate() {
            assert_eq!(table.get_int("foo"), Some(i as i64));
            assert_eq!(table.get_str("bar"), Some("blahblah".to_string()));
        }
    }

    if let Some(tables) = parsed.get("bar") {
        assert_eq!(tables.len(), 5);
        for (i, table) in tables.iter().enumerate() {
            let i_f32 = i as f32;
            assert_eq!(table.get_float("fizz"), Some(i_f32));
            assert_eq!(table.get_vec3("buzz"), Some(vec3(i_f32, i_f32, i_f32)));
        }
    }
}
