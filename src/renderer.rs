extern crate tera;
extern crate serde;
extern crate serde_json;

use std::collections::HashMap;
use self::tera::{Tera, Result, Value, try_get_value, to_value, from_value, GlobalFn};

use std::io::prelude::*; // to read from file

use thumbnail;
use data;
use assets;

fn string_from_file(path: std::path::PathBuf) -> String {
  let mut file = std::fs::File::open(path).unwrap();
  let mut string = String::new();
  file.read_to_string(&mut string).unwrap();
  string.clone()
}

pub fn load_content_types () -> std::collections::HashMap<String, ContentType> {

  // TODO: load content type editor js/css information from files? content type renderers cannot be read from files, if we want to render from inside rust. We could output code that renders in JS, though.
  let content_type_string = ContentType {
    editor_above: None,
    editor_below: Some(r#"<script>
//https://medium.com/@albertogasparin/getting-plain-text-from-user-input-on-a-contenteditable-element-b711aba2cb36
function getTextFromContenteditable(element) {
  let firstTag = element.firstChild.nodeName;
  let keyTag = new RegExp(
    firstTag === '#text' ? '<br' : '</' + firstTag,
    'i'
  );
  let tmp = document.createElement('p');
  tmp.innerHTML = element.innerHTML
    .replace(/<[^>]+>/g, function (m, i) {return (keyTag.test(m) ? '{ß®}' : '');})
    .replace(/{ß®}$/, '');
  return tmp.innerText.replace(/{ß®}/g, '\n');
}
    
document.addEventListener("DOMContentLoaded", function () {
  console.log("Initializing string editors!");
  Array.from(document.querySelectorAll('[data-editable=string]')).forEach(function (el) {
    console.log(["intializing string", el]);
    var config = { attributes: false, childList: true, characterData: true, subtree: true };
    var callback = function(records, observer) {
      console.log(["mutated", el]);
      window.updateValue(el.getAttribute("data-fieldname"), getTextFromContenteditable(el).trim());
    };
    var observer = new MutationObserver(callback);
    observer.observe(el, config);
  });
});</script>
    "#.to_string()),
    editable_template: r#"<div data-editable='string' contenteditable data-fieldname='{name}'>{data}</div>"#.to_string(),
  };
  let content_type_quill = ContentType {
    editor_above: Some(r#"
  <!-- Main Quill library -->
<script src="//cdn.quilljs.com/1.3.6/quill.min.js"></script>

<!-- Theme included stylesheets -->
<link href="//cdn.quilljs.com/1.3.6/quill.snow.css" rel="stylesheet">
<link href="//cdn.quilljs.com/1.3.6/quill.bubble.css" rel="stylesheet">

  <style>
  .ql-editor {
  padding: 12px;
  height: auto;
  }

  .ql-container {
  height: auto;
  font-family: inherit;
  font-size: inherit;
  }
  
  .ql-video > iframe {
    width:600px;
    height:400px;
  }
  </style>
      "#.to_string()),
    editor_below: Some(r#"<script>    
document.addEventListener("DOMContentLoaded", function () {

  const toolbarOptions = [
    //[{ font: [] }],
    [{ header: 1 }, { header: 2 }],
    //[{ header: [1, 2, 3, 4, 5, 6, false] }],
    //[{ align: [] }],
    [{ 'size': ['small', false, 'large', 'huge'] }],
    ['bold', 'italic', 'underline', 'strike'],
    ['link'],
    //['blockquote', 'code-block'],
    //[{ script: 'sub' }, { script: 'super' }],
    //[{ indent: '-1' }, { indent: '+1' }],
    //[{ color: [] }, { background: [] }],
    ['video'],
    ['image'],
  ];
  
  // TODO: make a new Box<tera::Function>repository for quill customizations used in this editor, and/or extract the js editor altogether from the rust code
    
  console.log("Initializing quill editors!");
  Array.from(document.querySelectorAll('[data-editable=quill]')).forEach(function (el) {
      console.log(["intializing quill", el]);
      let fieldname = el.getAttribute("data-fieldname");
      let quill = new Quill(el, {
        modules: {
          toolbar: toolbarOptions,
        },
        theme: 'bubble'
      });
      
      console.log(["quill", quill]);
      
      quill.setContents(window.data["_"+fieldname]);
            
      quill.on('text-change', function(delta, oldDelta, source) {
          console.log(["text-change", window.data]);
          window.updateValue(fieldname, quill.root.innerHTML);
          window.updateValue("_"+fieldname,quill.getContents());
      });
  });
});
</script>"#.to_string()),
    editable_template: r#"<div data-editable='quill' data-fieldname='{name}'>{data}</div>"#.to_string()
  };

  let mut content_types = HashMap::new();
  content_types.insert("string".to_string(), content_type_string);
  content_types.insert("quill".to_string(), content_type_quill);
  content_types
}



fn make_content_function () -> GlobalFn {
  Box::new(move |args: HashMap<String, Value>| -> Result<Value> {
  
    // TODO: maybe replace content filter/function altogether and prerender all the content fields and put them in the context?
  
    // TODO: find out how to pass content type information instead of loading again all the time
    let content_types: HashMap<String, ContentType> = load_content_types();
  
    // TODO: test is_boolean before
    let editable = args.get("editable").unwrap().as_bool().unwrap();

    let data = from_value::<String>(args.get("data").unwrap().clone()).unwrap();
    
    let r = match editable {
      true => {
        let name = from_value::<String>(args.get("name").unwrap().clone()).unwrap();
        
        let content_type = from_value::<String>(args.get("content_type").unwrap().clone()).unwrap();
        let ct = content_types.get(&content_type).unwrap();
        
        let result: Value = render_content_type(ct, editable, data, name).unwrap();
        Ok(result)
      },
      false => {
        Ok(to_value(data).unwrap())
      },
    };
    r
  })
}

fn make_data_function (path: std::path::PathBuf) -> GlobalFn {
  Box::new(move |args: HashMap<String, Value>| -> Result<Value> {
    match args.get("name") {
      Some(n) => Ok(to_value(data::load(&path, &args)).unwrap()),
      None => Err("'name' parameter missing.".into()),
    }
  })
}

fn filter_split (value: Value, args: HashMap<String, Value>) -> Result<Value> {
  let s = try_get_value!("filter_split", "value", String, value);
  let delim = match args.get("delim") {
    Some(d) => d.as_str().unwrap_or(","),
    None => ","
  };
  let result: Vec<&str> = s.split(delim).collect();
  
  Ok(to_value(result).unwrap())
}

fn filter_thumbnail (value: Value, args: HashMap<String, Value>) -> Result<Value> {
  let s = try_get_value!("filter_thumbnail", "value", String, value);
  // TODO
  let width = args.get("width").unwrap();
  let height = args.get("height").unwrap();
  
  let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).expect("SYSTEM TIME ERROR").as_secs();
  
  let path_string = format!("/thumbnails/{}-{}-{}", width, height, &s);
  
  let new_thumbnail = thumbnail::Thumbnail {
    src: std::path::PathBuf::from(s.clone()),
    width: width.as_u64().unwrap() as u32,
    height: height.as_u64().unwrap() as u32,
    url: path_string.clone(),
  };
  tmp.lock().unwrap().thumbnails.insert(format!("{}",&new_thumbnail), new_thumbnail);
  
  Ok(to_value(format!("{}?t={}",path_string,now)).unwrap())
}

fn filter_asset (value: Value, args: HashMap<String, Value>) -> Result<Value> {
  let s = try_get_value!("filter_asset", "value", String, value);

  let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).expect("SYSTEM TIME ERROR").as_secs();
  let path_string = format!("/assets/{}",&s);
  
  tmp.lock().unwrap().assets.push(
    assets::Asset {
      src: std::path::PathBuf::from(&s.clone()),
      url: path_string.clone(),
    });
  
  Ok(to_value(format!("{}?t={}",path_string,now)).unwrap())
}


#[derive(Debug)]
#[derive(Clone)]
pub struct TeraRenderer {
  path: std::path::PathBuf,
}

impl TeraRenderer {
  pub fn new(p: &std::path::Path) -> Result<TeraRenderer> {
    Ok(TeraRenderer {
      path: std::path::PathBuf::from(p)
    })
  }
}

#[derive(Debug)]
pub struct Config {
  pub content_types: HashMap<String, ContentType>,
  pub template: String,
  pub editable: bool,
}

#[derive(Debug, Clone)]
pub struct ContentType {
  editor_above: Option<String>,
  editor_below: Option<String>,
  editable_template: String,
}


fn render_content_type(content_type: &ContentType, editable: bool, data: String, name: String) -> Result<Value> {
  match editable {
    true => Ok(to_value(content_type.editable_template.replace("{name}", &name).replace("{data}", &data)).unwrap()),
    false => Ok(to_value(data).unwrap()),
  }
}

impl std::fmt::Display for Document {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "sizeof Html: {}\n", self.html.len())?;
        write!(f, "Assets:\n")?;
        for v in &self.assets {
            write!(f, "{},", v)?;
        }
        write!(f, "Thumbnails:\n")?;
        for (k, v) in &self.thumbnails {
            write!(f, "{},", v)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Document {
  pub url: String,
  pub html: String,
  pub thumbnails: HashMap<String, thumbnail::Thumbnail>,
  pub assets: Vec<assets::Asset>
}


struct Tmp {
  thumbnails: HashMap<String,thumbnail::Thumbnail>,
  assets: Vec<assets::Asset>
}

lazy_static! {
  static ref tmp: std::sync::Mutex<Tmp> = std::sync::Mutex::new(Tmp { thumbnails: HashMap::new(), assets: [].to_vec() });
}

fn clear_tmp() -> () {
  let mut t = tmp.lock().unwrap();
  t.thumbnails.clear();
  t.assets.clear();
}

pub trait Renderer {
  fn render(&self, config: Config, data: serde_json::Value, url: String) -> Result<Document>;
}

impl Renderer for TeraRenderer {
  fn render(&self, config: Config, data: serde_json::Value, url: String) -> Result<Document> {
    println!("Rendering template: {}", &config.template);
  
    let mut t =  Tera::new(&self.path.join("templates/**/*.tera").to_str().unwrap()).unwrap();
    t.register_filter("asset", filter_asset);
    t.register_filter("split", filter_split);
    t.register_function("content", make_content_function());
    t.register_function("data", make_data_function(self.path.to_path_buf()));
    t.register_filter("thumbnail", filter_thumbnail);
    
    let mut above = String::new();
    let mut below = String::new();

    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).expect("SYSTEM TIME ERROR").as_secs();
  
    above = format!(r#"{}<link rel="stylesheet" type="text/css" href="/main.css?t={}">"#, above, &now);
    
    if config.editable {
      for (name, ct) in &config.content_types {
        let a = ct.clone(); // TODO: find out how to do this without cloning
        above = above + &a.editor_above.unwrap_or("".to_string());
        below = below + &a.editor_below.unwrap_or("".to_string());
      }
    
      above = above + &format!(r#"
<script src='/domvm.full.js'></script>
<script>
window.data = {data};
{js}</script>
    "#, data=data.to_string(), js=string_from_file(std::path::PathBuf::from("./assets/editPage.js")));
      below = below + r#"
<style>
*[data-editable] {
    background: yellow !important;
}
</style>
      "#;
    }
  
    let mut context = tera::Context::new();
    let emptystring = json!("");
    context.insert("meta_title", &data.get("meta_title").unwrap_or(&emptystring));
    context.insert("meta_description", &data.get("meta_description").unwrap_or(&emptystring));
    context.insert("editable", &config.editable);
    context.insert("path", &url);
    context.insert("data", &data);
    context.insert("above", &above);
    context.insert("below", &below);
    
    clear_tmp();
    match t.render(&config.template, &context) {
      Ok(html) => {
        let mut t = tmp.lock().unwrap();
        Ok(Document {
          thumbnails: t.thumbnails.clone()
          , assets: t.assets.clone()
          , html: html
          , url: url
        })
      },
      Err(e) => {
        println!("Error: {:?}", e);
        clear_tmp();
        Err(e)
      },
    }
  }
}
