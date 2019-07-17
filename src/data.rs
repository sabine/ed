use paths;
use utils;
//use csv;


/*pub struct Data {
  pub data: serde_json::Value
}*/

/*
pub trait DataSource {
  fn load(&self, args: &std::collections::HashMap<String, serde_json::Value>) -> serde_json::Value;
}*/

fn get_images(path: &std::path::Path, folder: &std::path::Path) -> serde_json::Value {
  println!("get images from {:?}", folder);
  let files = std::fs::read_dir(&path.join(folder)).unwrap();
  let mut images = Vec::new();
  
  let image_extensions = vec!["png".to_string(), "jpg".to_string(), "gif".to_string(), "bmp".to_string()];
  
  files
    .filter_map(Result::ok)
    .filter(|f| { f.file_type().unwrap().is_file() })
    .filter(|f| { image_extensions.iter().find(|ext| *ext == f.path().extension().unwrap().to_str().unwrap()).is_some() })
    .for_each(|f| {
      let img = json!({
        "path": f.path().clone(),
        "filename": f.file_name().to_str().unwrap(),
      });
      images.push(img);
    });
    // TODO: sort out images since giving this to the thumbnail filter doesn't work
    
  println!("get_images: {:?}", images);
  json!(images)
}


#[derive(Debug, Deserialize, Eq, PartialEq)]
struct CsvProdukt {
  id: String,
  name: String,
  price: u32,
  images: String,
  itemtype: String,
  categories: String,
  description: String
}

#[derive(Debug, Serialize, Eq, PartialEq)]
struct Image {
  src: String,
  small_thumbnail: String,
  large_thumbnail: String
}

#[derive(Debug, Eq, PartialEq, Serialize)]
struct Produkt {
  id: String,
  name: String,
  price: u32,
  images: Vec<Image>,
  itemtype: String,
  categories: Vec<String>,
  description: String
}

//id,name,price,images,itemtype,categories,description

pub fn load_csv(path: &std::path::Path, args: &std::collections::HashMap<String, serde_json::Value>) -> serde_json::Value {
  let folder = args.get("name").unwrap().as_str().unwrap();
  let filename = &paths::data_path(&path).join(folder).join("data.csv");
  println!("data file name: {:?}", filename);
  let reader = std::fs::File::open(filename).unwrap();
  
  let mut rdr = csv::Reader::from_reader(reader);
  let mut data = Vec::new();

  for result in rdr.deserialize() {
    println!("r: {:?}", result);
    let record: CsvProdukt = result.unwrap();
    let p: Produkt = Produkt {
      id: record.id,
      name: record.name,
      price: record.price,
      images: record.images.split(",").map(|s| Image { 
      // TODO: generate small and large thumbnails
        src: s.to_string(),
        small_thumbnail: s.to_string(),
        large_thumbnail: s.to_string()}).collect(),
      itemtype: record.itemtype,
      categories: record.categories.split(",").map(|s| s.to_string()).collect(),
      description: record.description };
    data.push(serde_json::to_value(p).unwrap());  
  }
  
  println!("CSV data: {:?}", data);
  
  serde_json::value::Value::Array(data)
}

pub fn load_references(path: &std::path::Path, args: &std::collections::HashMap<String, serde_json::Value>) -> serde_json::Value {
  let references = std::fs::read_dir(&paths::data_path(&path).join("references")).unwrap();
  
  println!("Trying to load references..\n{:?}", references);
  
  let mut result = Vec::new();
  
  references
    .filter_map(Result::ok)
    .filter(|r| { r.file_type().unwrap().is_dir() })
    .for_each(|r| {
      let p = r.path().join("data.json");
      println!("Reading from {:?}", p);
      let o = utils::read_json_from_file(&p);
      
      let b = json!({
        "id": r.path(),
        "before": get_images(&path, &r.path().strip_prefix(&path).unwrap().join("before")),
        "after": get_images(&path, &r.path().strip_prefix(&path).unwrap().join("after")),
        "data": o,
      });
      result.push(b);
    });
    
  println!("data: {:?}", result);
  
  json!(result)
  
  // TODO: move used images to target folder
}

pub fn load_products(path: &std::path::Path, args: &std::collections::HashMap<String, serde_json::Value>) -> serde_json::Value {
  let name = args.get("name").unwrap().as_str().unwrap().to_string();

  let products = std::fs::read_dir(&paths::data_path(&path).join(name)).unwrap();
  
  println!("Trying to load products..\n{:?}", products);
  
  let mut result = Vec::new();
  
  products
    .filter_map(Result::ok)
    .filter(|r| { r.file_type().unwrap().is_dir() })
    .for_each(|r| {
      let p = r.path().join("data.json");
      println!("Reading from {:?}", p);
      let o = utils::read_json_from_file(&p);
      
      let b = json!({
        "id": r.path(),
        "images": get_images(&path, &r.path().strip_prefix(&path).unwrap()),
        "data": o,
      });
      result.push(b);
    });
    
  println!("data: {:?}", result);
  
  json!(result)
  
  // TODO: move used images to target folder
}


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
    "csv" => load_csv(path, args),
    "references" => load_references(path, args),
    "products" => load_products(path, args),
    "lua" => unimplemented!(),
      // TODO: allow lua scripts to load data?
    _ => unimplemented!(),
  }
}