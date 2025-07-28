/*
 * For saving data in a binary format
 * */

mod tests;

use cgmath::vec3;
use std::collections::HashMap;

use crate::game::entities::Vec3;

pub struct ByteStream {
    bytes: Vec<u8>,
    index: usize,
}

impl ByteStream {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self { bytes, index: 0 }
    }

    pub fn get_byte(&mut self) -> Option<u8> {
        if self.index >= self.bytes.len() {
            return None;
        }

        let byte = self.bytes[self.index];
        self.index += 1;
        Some(byte)
    }

    pub fn get_buf(&mut self, count: usize) -> Vec<u8> {
        let mut buf = Vec::with_capacity(count);
        for _ in 0..count {
            buf.push(self.get_byte().unwrap_or(0));
        }
        buf
    }

    pub fn at_end(&self) -> bool {
        self.index >= self.bytes.len()
    }
}

pub enum DataType {
    Int(i64),
    Float(f32),
    Vec3(Vec3),
    Str(String),
}

impl DataType {
    pub fn get_type_flag(&self) -> u8 {
        match self {
            Self::Int(_) => 1,
            Self::Float(_) => 2,
            Self::Vec3(_) => 3,
            Self::Str(_) => 4,
        }
    }
}

pub struct DataTable {
    values: HashMap<String, DataType>,
}

#[allow(dead_code)]
impl DataTable {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn add_int(&mut self, name: &str, val: i64) {
        self.values.insert(name.to_string(), DataType::Int(val));
    }

    pub fn add_float(&mut self, name: &str, val: f32) {
        self.values.insert(name.to_string(), DataType::Float(val));
    }

    pub fn add_vec3(&mut self, name: &str, val: Vec3) {
        self.values.insert(name.to_string(), DataType::Vec3(val));
    }

    pub fn add_str(&mut self, name: &str, val: &str) {
        self.values
            .insert(name.to_string(), DataType::Str(val.to_string()));
    }

    pub fn get_int(&self, name: &str) -> Option<i64> {
        let val = self.values.get(name)?;
        if let DataType::Int(v) = val {
            Some(*v)
        } else {
            None
        }
    }

    pub fn get_float(&self, name: &str) -> Option<f32> {
        let val = self.values.get(name)?;
        if let DataType::Float(v) = val {
            Some(*v)
        } else {
            None
        }
    }

    pub fn get_vec3(&self, name: &str) -> Option<Vec3> {
        let val = self.values.get(name)?;
        if let DataType::Vec3(v) = val {
            Some(*v)
        } else {
            None
        }
    }

    pub fn get_str(&self, name: &str) -> Option<String> {
        let val = self.values.get(name)?;
        if let DataType::Str(v) = val {
            Some(v.to_string())
        } else {
            None
        }
    }

    pub fn add_int_from_stream(&mut self, stream: &mut ByteStream) {
        let name = get_str_from_stream(stream);
        if name.is_empty() {
            return;
        }
        let int_bytes = stream.get_buf(8);
        let mut buf = [0u8; 8];
        buf.copy_from_slice(&int_bytes);
        let val = i64::from_be_bytes(buf);
        self.values.insert(name, DataType::Int(val));
    }

    pub fn add_float_from_stream(&mut self, stream: &mut ByteStream) {
        let name = get_str_from_stream(stream);
        if name.is_empty() {
            return;
        }
        let float_bytes = stream.get_buf(4);
        let mut buf = [0u8; 4];
        buf.copy_from_slice(&float_bytes);
        let val = f32::from_be_bytes(buf);
        self.values.insert(name, DataType::Float(val));
    }

    pub fn add_vec3_from_stream(&mut self, stream: &mut ByteStream) {
        let name = get_str_from_stream(stream);
        if name.is_empty() {
            return;
        }
        let vec_bytes = stream.get_buf(4);
        let mut buf = [0u8; 4];
        buf.copy_from_slice(&vec_bytes);
        let x = f32::from_be_bytes(buf);

        let vec_bytes = stream.get_buf(4);
        let mut buf = [0u8; 4];
        buf.copy_from_slice(&vec_bytes);
        let y = f32::from_be_bytes(buf);

        let vec_bytes = stream.get_buf(4);
        let mut buf = [0u8; 4];
        buf.copy_from_slice(&vec_bytes);
        let z = f32::from_be_bytes(buf);

        self.values.insert(name, DataType::Vec3(vec3(x, y, z)));
    }

    pub fn add_str_from_stream(&mut self, stream: &mut ByteStream) {
        let name = get_str_from_stream(stream);
        if name.is_empty() {
            return;
        }
        let val = get_str_from_stream(stream);
        self.values.insert(name, DataType::Str(val));
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];

        for (name, val) in &self.values {
            let type_flag = val.get_type_flag();
            bytes.push(type_flag);

            bytes.extend((name.len() as u32).to_be_bytes());
            let name_bytes: Vec<u8> = name.bytes().collect();
            bytes.extend(name_bytes);

            match val {
                DataType::Int(int) => bytes.extend(int.to_be_bytes()),
                DataType::Float(float) => bytes.extend(float.to_be_bytes()),
                DataType::Vec3(v) => {
                    bytes.extend(v.x.to_be_bytes());
                    bytes.extend(v.y.to_be_bytes());
                    bytes.extend(v.z.to_be_bytes());
                }
                DataType::Str(str) => {
                    bytes.extend((str.len() as u32).to_be_bytes());
                    let val_bytes: Vec<u8> = str.bytes().collect();
                    bytes.extend(val_bytes);
                }
            }
        }

        //Stop
        bytes.push(0);

        bytes
    }
}

fn get_str_from_stream(stream: &mut ByteStream) -> String {
    let len_bytes = stream.get_buf(4);
    if stream.at_end() {
        return "".to_string();
    }
    let mut buf = [0u8; 4];
    buf.copy_from_slice(&len_bytes);
    let len = u32::from_be_bytes(buf);
    match String::from_utf8(stream.get_buf(len as usize)) {
        Ok(s) => s,
        Err(msg) => {
            eprintln!("{msg}");
            "".to_string()
        }
    }
}

fn get_table(stream: &mut ByteStream) -> DataTable {
    let mut table = DataTable::new();
    loop {
        let type_flag = stream.get_byte();

        match type_flag.unwrap_or(0) {
            //0 = stop
            0 => return table,
            //1 = int
            1 => table.add_int_from_stream(stream),
            //2 = float
            2 => table.add_float_from_stream(stream),
            //3 = vec
            3 => table.add_vec3_from_stream(stream),
            //4 = str
            4 => table.add_str_from_stream(stream),
            _ => {}
        }
    }
}

fn get_data_list(stream: &mut ByteStream) -> Vec<DataTable> {
    let mut list = vec![];

    //Get length
    let len_bytes = stream.get_buf(4);
    if stream.at_end() {
        return list;
    }
    let mut buf = [0u8; 4];
    buf.copy_from_slice(&len_bytes);
    let len = u32::from_be_bytes(buf);

    for _ in 0..len {
        let table = get_table(stream);
        if table.values.is_empty() {
            return list;
        }
        list.push(table);
        if stream.at_end() {
            return list;
        }
    }

    list
}

pub type ParsedData = HashMap<String, Vec<DataTable>>;

pub fn parse_binary_data(stream: &mut ByteStream) -> ParsedData {
    let mut tables = HashMap::new();

    loop {
        let name = get_str_from_stream(stream);
        let list = get_data_list(stream);
        if list.is_empty() {
            return tables;
        }
        tables.insert(name, list);
        if stream.at_end() {
            return tables;
        }
    }
}

pub fn get_table_list_bytes(name: &str, tables: &[DataTable]) -> Vec<u8> {
    let mut bytes = vec![];

    if tables.is_empty() {
        return bytes;
    }

    bytes.extend((name.len() as u32).to_be_bytes());
    let name_bytes: Vec<u8> = name.bytes().collect();
    bytes.extend(name_bytes);
    bytes.extend((tables.len() as u32).to_be_bytes());
    for data_table in tables {
        bytes.extend(data_table.to_bytes());
    }
    bytes
}
