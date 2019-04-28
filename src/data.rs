use paths;
use utils;

/*
pub trait DataSource {
  fn load(&self, args: &std::collections::HashMap<String, serde_json::Value>) -> serde_json::Value;
}*/

pub fn load (path: &std::path::Path, args: &std::collections::HashMap<String, serde_json::Value>) -> serde_json::Value {
  // TODO: implement data source lookup
  match args.get("type").unwrap().as_str().unwrap() {
    "json-file" => {
      let name = args.get("name").unwrap().as_str().unwrap().to_string();
      let p = &paths::data_path(&path).join(name+".json");
      let json_string = utils::read_string_from_file(&p);
      let json: serde_json::Value = serde_json::from_str(&json_string).expect("JSON was not well-formatted");
    
      json
    },
    _ => unimplemented!(),
  }
}