use std::io::Read;

pub fn read_string_from_file(p: &std::path::Path) -> String {
  let mut datafile = std::fs::File::open(&p).unwrap();
  let mut datastring = String::new();
  datafile.read_to_string(&mut datastring).unwrap();
  datastring
}

pub fn read_json_from_file (p: &std::path::Path) -> serde_json::Value { 
  serde_json::from_str(&read_string_from_file(&p)).expect("JSON was not well-formatted")
}