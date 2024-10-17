mod tests;

use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::iter::Peekable;
use std::str::Chars;

/*
 * An entry, how an impfile is organized
 * */
pub struct Entry {
    name: String,
    variables: HashMap<String, String>,
}

pub type EntryList = Vec<Entry>;

impl Entry {
    pub fn new(entry_name: &str) -> Self {
        Self {
            name: entry_name.to_string(),
            variables: HashMap::new(),
        }
    }

    pub fn from_vec(entry_name: &str, vars: Vec<(String, String)>) -> Self {
        let mut variable_map = HashMap::new();
        for (name, value) in vars {
            variable_map.insert(name, value);
        }

        Self {
            name: entry_name.to_string(),
            variables: variable_map,
        }
    }

    //Returns the name of the entry
    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    //Returns a variable value, if a variable is not found "" is returned
    pub fn get_var(&self, var_name: &str) -> String {
        self.variables
            .get(var_name)
            .cloned()
            .unwrap_or("".to_string())
    }

    pub fn add_string(&mut self, var_name: &str, s: &str) {
        self.variables.insert(var_name.to_string(), s.to_string());
    }

    pub fn add_float(&mut self, var_name: &str, v: f32) {
        self.variables.insert(var_name.to_string(), v.to_string());
    }

    pub fn add_bool(&mut self, var_name: &str, b: bool) {
        self.variables.insert(var_name.to_string(), b.to_string());
    }

    pub fn add_integer(&mut self, var_name: &str, i: i64) {
        self.variables.insert(var_name.to_string(), i.to_string());
    }

    //Converts the value into a formatted string, used for serializing to a file
    pub fn to_impfile_string(&self) -> String {
        let mut res = String::new();
        //name
        res.push('\"');
        res.push_str(&self.name);
        res.push_str("\" {\n");
        //variables
        for (var_name, val) in &self.variables {
            res.push_str("\t\"");
            res.push_str(var_name);
            res.push_str("\" = \"");
            res.push_str(val);
            res.push_str("\";\n");
        }
        res.push('}');

        res
    }
}

//Returns Ok if quotes are valid, returns an Err if quotes are invalid
fn validate_quotes(line: &str) -> Result<(), String> {
    let quote_count = line.chars().filter(|ch| *ch == '\"').count();

    if quote_count % 2 != 0 {
        return Err("Mismatched quotes".to_string());
    }

    Ok(())
}

//Removes any comments from a string
fn strip_comment(line: &str) -> String {
    let mut res = String::new();
    for ch in line.chars() {
        if ch == '#' {
            return res;
        }

        res.push(ch);
    }

    res
}

//Removes any whitespace from a line
fn strip_whitespace(line: &str) -> String {
    let mut res = String::new();
    let mut quote_count = 0;
    for ch in line.chars() {
        if ch.is_whitespace() && quote_count % 2 == 0 {
            continue;
        }

        if ch == '\"' {
            quote_count += 1;
        }

        res.push(ch);
    }

    res
}

//Writes a comment into a file
pub fn write_comment(outfile: &mut File, comment_text: &str) {
    let mut text = String::new();
    for ch in comment_text.chars() {
        if ch == '\n' {
            let mut comment_text = String::new();
            comment_text.push_str("# ");
            comment_text.push_str(&text);
            comment_text.push('\n');

            let res = outfile.write_all(comment_text.as_bytes());
            //Output any potential errors
            if let Err(msg) = res {
                eprintln!("E: {msg}");
            }
            text.clear();
            continue;
        }

        text.push(ch);
    }

    let mut comment_text = String::new();
    comment_text.push_str("# ");
    comment_text.push_str(&text);
    comment_text.push('\n');
    let res = outfile.write_all(comment_text.as_bytes());
    //Output any potential errors
    if let Err(msg) = res {
        eprintln!("E: {msg}");
    }
}

//Returns either Ok(name) or Err(msg)
fn parse_name(file_chars: &mut Peekable<Chars>) -> Result<String, String> {
    let mut name = String::new();

    let mut quote_count = 0;
    let mut ch = file_chars.next();
    while quote_count < 2 && ch.is_some() {
        if ch.unwrap_or('\0') == '\"' {
            quote_count += 1;
            if quote_count < 2 {
                ch = file_chars.next();
            }
            continue;
        }

        //Inside a quote
        if quote_count == 1 {
            if let Some(ch) = ch {
                name.push(ch);
            }
        }

        ch = file_chars.next();
    }

    //Empty name is invalid, return error
    if name.is_empty() {
        return Err("Failed to parse name!".to_string());
    }

    Ok(name)
}

//Returns Ok((name, value)) or Err(msg)
fn parse_variable(file_chars: &mut Peekable<Chars>) -> Result<(String, String), String> {
    // 0 = name, 1 = value
    let mut var = [String::new(), String::new()];
    let mut readinto = 0;
    let mut ch = file_chars.next();
    let mut quote_count = 0;
    while !(ch.unwrap_or('\0') == ';' && quote_count % 2 == 0) && ch.is_some() {
        //Count quotes
        if ch.unwrap_or('\0') == '\"' {
            quote_count += 1;
            ch = file_chars.next();
            continue;
        }

        if ch.unwrap_or('\0') == '=' && readinto == 0 && quote_count == 2 {
            readinto = 1;
            ch = file_chars.next();
            continue;
        } else if ch.unwrap_or('\0') == '=' && quote_count % 2 == 1 {
            //Inside a string
            var[readinto].push('=');
            ch = file_chars.next();
            continue;
        } else if ch.unwrap_or('\0') == '=' {
            //Syntax error
            return Err("Extra \'=\' found".to_string());
        }

        //Syntax error
        if quote_count % 2 == 0 {
            return Err("Extra characters outside of quotes".to_string());
        }

        if let Some(ch) = ch {
            var[readinto].push(ch)
        }

        ch = file_chars.next();
    }

    if var[0].is_empty() {
        return Err("Failed to parse variable!".to_string());
    }

    Ok((var[0].clone(), var[1].clone()))
}

fn parse_entry(file_chars: &mut Peekable<Chars>) -> Result<Entry, String> {
    let name = parse_name(file_chars)?;
    let mut ch = file_chars.next();
    if ch.unwrap_or('\0') != '{' {
        return Err("No opening {".to_string());
    }

    let mut entry = Entry::new(&name);
    let mut entry_content = String::new();
    ch = file_chars.next();
    while ch.is_some() && ch.unwrap_or('\0') != '}' {
        if let Some(ch) = ch {
            entry_content.push(ch);
        }
        ch = file_chars.next();
    }

    let mut entry_content_chars = entry_content.chars().peekable();
    while entry_content_chars.peek().is_some() {
        match parse_variable(&mut entry_content_chars) {
            Ok((name, value)) => {
                entry.variables.insert(name, value);
            }
            Err(msg) => {
                let err = format!("Error in \'{}\': ", entry.name);
                return Err(err.to_string() + &msg);
            }
        }
    }

    Ok(entry)
}

pub fn parse_file(path: &str) -> EntryList {
    let mut entries = vec![];

    let file = File::open(path);
    let mut file_contents = String::new();
    match file {
        Ok(mut file) => {
            let res = file.read_to_string(&mut file_contents);
            match res {
                Ok(sz) => eprintln!("read {sz} bytes from {path}"),
                Err(msg) => eprintln!("E: {msg}"),
            }
        }
        Err(msg) => {
            eprintln!("{msg}");
        }
    }

    //Strip lines
    let file_lines: Vec<String> = file_contents
        .lines()
        .map(|line| strip_whitespace(&strip_comment(line)))
        .collect();

    //Check for syntax errors
    for (line_num, line) in file_lines.iter().enumerate() {
        let res = validate_quotes(line);
        if let Err(msg) = res {
            eprintln!("Syntax error on line: {line_num} in {path}");
            eprintln!("E: {msg}");
            return entries;
        }
    }

    //Parse contents
    let file_lines_concat = file_lines.concat();
    let mut file_chars = file_lines_concat.chars().peekable();
    while file_chars.peek().is_some() {
        let entry = parse_entry(&mut file_chars);
        match entry {
            Ok(entry) => {
                entries.push(entry);
            }
            Err(msg) => {
                eprintln!("Syntax error in {path}.");
                eprintln!("{msg}");
                return entries;
            }
        }
    }

    entries
}
