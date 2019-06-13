#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate notify;
use notify::{RecommendedWatcher, Watcher, RecursiveMode, DebouncedEvent};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;

#[macro_use]
extern crate rouille;

extern crate image;

extern crate walkdir;
extern crate ftp;

extern crate web_view;
use web_view::*;

use std::thread;
use std::fs::File;
//use std::error::Error;
use std::io::prelude::*;

mod utils;
mod thumbnail;
mod assets;
mod renderer;
use renderer::{TeraRenderer, Renderer, ContentType, load_content_types};
mod css;
mod paths;
mod ftp_upload;
mod data;

use paths::*;



#[derive(Debug)]
enum Page{
  MainPage,
  ProjectPage { path: std::path::PathBuf, config: serde_json::value::Value, pages: Vec<String> },
  EditPage {path: std::path::PathBuf, filename: String, rendered: renderer::Document},
}

#[derive(Deserialize)]
#[derive(Serialize)]
#[serde(tag = "cmd")]
pub enum Cmd {
	OpenProject,
  RenderProject,
  RenderAndUpload,
  CloseProject,
  EditPage { filename: String },
  SavePage { data: serde_json::value::Value },
  ReturnToProject,
}

struct Editor {
  current_page: Page,
  content_types: std::collections::HashMap<String, ContentType>,
  watcher: RecommendedWatcher,
  //watch_path_sender: Sender<std::path::PathBuf>,
}

fn js_invoke (cmd: Cmd) -> String {
  format!("external.invoke(JSON.stringify({cmd}))", cmd=serde_json::to_string(&cmd).unwrap())
}

fn js_button (cmd: Cmd, text: &str) -> String {
  format!("<button onclick=\"{cmd}\">{text}</button>", cmd=js_invoke(cmd).replace("\"", "'"), text=text)
}

fn url_from_filename (filename: &str) -> String {
  if filename == "index" {
    "/".to_string()
  } else {
    filename.replace("/index", "/")
  }
}


fn set_html(webview: &mut WebView<'_,()>, s: &str) -> () {
  let html = &format!("{}{}", "document.documentElement.innerHTML=", escape(s));
  webview.eval(html);
}

fn render(path: &std::path::Path, page: String, url: String, editable: bool) -> renderer::Document {
  let r = TeraRenderer::new(path).unwrap();

  let json_filename = page.clone()+".json";
  let tera_filename = page.clone()+".tera";

  let p = path.join("pages").join(json_filename);
  println!("Path: {}\n", path.to_string_lossy());
  println!("Datafile: {}\n", p.to_string_lossy());
  let json = utils::read_json_from_file(&p);
  
  println!("Rendering...");
  let renderer_config = renderer::Config {
    content_types: load_content_types(),
    template: (tera_filename).to_string(),
    path: url,
    editable: editable,
    };
    
  let result = r.render(renderer_config, json).unwrap();
  result
}

fn render_page (path: &std::path::Path, name: &str) -> () {
  let result = render(path, name.to_string(), url_from_filename(&name).to_string(), false);
  
  println!("{}", result);
  println!("creating output_path: {:?}", output_path(&path).join(name));
  
  std::fs::create_dir_all(output_path(&path).join(name));
  println!("successful");
  
  let mut file = File::create(output_path(path).join(name.to_owned() +".html")).unwrap();
  file.write_all(&result.html.as_bytes());
  
  for thumbnail in result.thumbnails {
    thumbnail::render_thumbnail(&path, &thumbnail);
  }
}

fn load_pages (p: &std::path::Path) -> Vec<String> {
  let mut pages: Vec<String> = Vec::new();
              
  for entry in walkdir::WalkDir::new(&pages_path(&p)) {
    let e = entry.unwrap();
    if e.file_type().is_file() && e.path().extension() == Some(std::ffi::OsStr::new("json")) {
      println!("{}", e.path().display());
      pages.push(e.path().to_str().unwrap().replace(p.join("pages").to_str().unwrap(), "").replace(".json", "").replace("\\","/").replacen("/","",1));
    }
  }
  pages
}

fn load_project_config(p: &std::path::Path) -> serde_json::value::Value {
  println!("load_project_config {}\n", p.display());
  let path = p.clone().join("project.json");
  let mut file = File::open(path).unwrap();
  let mut string = String::new();
  file.read_to_string(&mut string).unwrap();

  let json = serde_json::from_str(&string).expect("project configuration was not well-formatted JSON");
  println!("result: {}\n", json);
  json
}

fn render_project (path: &std::path::Path, pages: &Vec<String>) -> () {
  println!("rendering project: {:?}", path);
  std::fs::remove_dir_all(output_path(&path));
  
  for page in pages {
    render_page(path, page);
  }
  
  println!("copying assets:");
  for entry in walkdir::WalkDir::new(&asset_path(&path)) {
    let e = entry.unwrap();
    if e.file_type().is_file() {
      let subpath = e.path().strip_prefix(asset_path(&path)).unwrap();
      let source = e.path();
      println!("source: {:?}", source);
      let target = output_path(&path).join(subpath);
      println!("target: {:?}", target);
      std::fs::create_dir_all(&target.parent().unwrap());
      std::fs::copy(source, target);
    }
  }
}

impl Editor {
  fn route_to(&mut self, webview: &mut WebView<'_,()>, new_page: Page) -> () {
    println!("Route To: {:?}", new_page);
  
    self.current_page = new_page;
    webview.eval("window.location=\"http://localhost:54321\";");
    ()
  }
  
  fn handle_external(&mut self, webview: &mut WebView<'_,()>, arg: &str) -> () {
    println!("Cmd: {}\n", &arg);
    use Cmd::*;
    
    match serde_json::from_str(arg).unwrap() {
      OpenProject => match webview.dialog().open_file("Projekt laden...", "").unwrap() {
            Some(path) => {
              let p = path.parent().unwrap();
              
              self.route_to(webview, Page::ProjectPage{ path:p.to_path_buf(), config: load_project_config(p), pages:load_pages(p)});
              self.watcher.watch(p.join("css/"), RecursiveMode::Recursive);
            },
            None => {
              webview
                .dialog()
                .warning("Warning", "Keine Datei gewaehlt.");
              ()
            },
      },
      RenderProject => {
        match self.current_page {
          Page::ProjectPage {ref path, ref pages, ..} => {
            render_project(path, pages);
          },
          _ => {
            panic!("Wrong page!");
          }
        }
      },
      RenderAndUpload => {
        match self.current_page {
          Page::ProjectPage {ref path, ref config, ref pages} => {
            render_project(path, pages);
            
            let ftp = config.get("ftp").unwrap();
            let host = ftp.get("host").unwrap().as_str().unwrap();
            let user = ftp.get("user").unwrap().as_str().unwrap();
            let pass = ftp.get("pass").unwrap().as_str().unwrap();
            let remote = match ftp.get("remote") {
              None => "/",
              Some(v) => v.as_str().unwrap(),
            };
            
            ftp_upload::upload (host, user, pass, remote, &output_path(&path));
          },
          _ => {
            panic!("Wrong page!");
          }
        } 
      },
      CloseProject => {
        let p = match self.current_page {
          Page::ProjectPage{ref path, ..} => {
            path.clone()
          },
          _ => {
            panic!("Wrong page!");
          }
        };
        self.watcher.unwatch(p.join("css/"));
        self.route_to(webview, Page::MainPage);
        ()
      },
      ReturnToProject => {
        let p = match self.current_page {
          Page::EditPage {ref path, ..} => {
            path.clone()
          },
          _ => {
            panic!("Wrong page!");
          }
        };
        self.route_to(webview, Page::ProjectPage {path: p.clone(), config: load_project_config(&p), pages: load_pages(&p)});
      },
      EditPage {filename} => {
        let p = match self.current_page {
          Page::ProjectPage{ ref path, .. } => {
            path.clone()
          },
          _ => {
            panic!("Wrong page!");
          }
        };
        let result = render(&p, filename.clone(), url_from_filename(&filename).to_string(), true);
        
        self.route_to(webview, Page::EditPage{path:p, filename:filename, rendered:result});
      },
      SavePage {data} => {
        match self.current_page {
          Page::EditPage{ref path, ref filename, ..} => {
            let datafilename = pages_path(path).join(filename).with_extension("json");
            println!("Save Page Data: {}\nTo: {:?}\n", data, datafilename);
          
            let mut file = File::create(datafilename).unwrap();
            file.write_all(data.to_string().as_bytes());
          },
          _ => {
            panic!("Wrong page!");
          }
        };
      },
    }
  }
}

fn get_path_from_editor(editor: &Editor) -> Option<std::path::PathBuf> {
  match editor.current_page {
    Page::MainPage => None,
    Page::ProjectPage {ref path, ..} => Some(path.to_path_buf()),
    Page::EditPage {ref path, ..} => Some(path.to_path_buf())
  }
}

fn main() {
  println!("Starting..."); 
  
  
  let (watch_sender, watch_receiver):(Sender<DebouncedEvent>, Receiver<DebouncedEvent>) = channel();
  
  let w: RecommendedWatcher = Watcher::new(watch_sender, Duration::from_secs(2)).unwrap();
  
  let editor: std::sync::Arc<std::sync::RwLock<Editor>> = std::sync::Arc::new(std::sync::RwLock::new(Editor {
    current_page: Page::MainPage,
    content_types: std::collections::HashMap::new(),
    watcher: w
  }));
  
  let rouille_editor = editor.clone();
  thread::spawn(move || {
    rouille::start_server("localhost:54321", move |request| {
      router!(request,
        (GET) (/) => {
          let html_string = match (*rouille_editor.read().unwrap()).current_page {
            Page::MainPage => format!(r#"
<!doctype html>
<html>
	<body>
		{cmd}
	</body>
</html>
"#, cmd=js_button(Cmd::OpenProject, "Projekt laden")),
            Page::ProjectPage {ref path, ref pages, ..} => {
              let mut edit:Vec<String> = Vec::new();
              
              for page in pages {
                edit.push(format!("<li>{}</li", js_button(Cmd::EditPage{filename:page.to_string()}, page )));
              }
              
              format!(r#"
          <!doctype html>
          <html>
            <body>
              <h1>{path}</h1>
              <ul>
              {edit}
              </ul>
              {render}
              <br/>
              {upload}
              <br/>
              {close}
            </body>
          </html>
            "#, path=path.to_string_lossy(), edit=edit.join(""), render=js_button(Cmd::RenderProject, "Erstellen"), upload=js_button(Cmd::RenderAndUpload, "Erstellen und hochladen"), close=js_button(Cmd::CloseProject, "Zur&uuml;ck"))
            },
          
            Page::EditPage {ref path, ref filename, ref rendered} => {
              rendered.html.clone()
            },
          };
          rouille::Response::html(html_string).with_no_cache()
        },
        _ => {
          match rouille_editor.read().unwrap().current_page {
            Page::EditPage{ref path, ref filename, ref rendered} => {
              let response = rouille::match_assets(&request, &asset_path(&path));
              if response.is_success() {
                return response.with_no_cache();
              } else {
                let editor_response = rouille::match_assets(&request, &asset_path(std::path::Path::new(".")));
                if editor_response.is_success() {
                  return editor_response.with_no_cache();
                } else {
                  rouille::Response::empty_404()
                }
              }
            },
            _ => rouille::Response::empty_404(),
          }
        }
      )
    });
  });

  fn compile (path: &std::path::Path, event: &DebouncedEvent, sender: &Sender<()>) -> () {
    fn c (path: &std::path::Path, sender: &Sender<()>) -> () {
      css::compile_sass(&path, &asset_path(&path));
      sender.send(());
    }
  
    match event {
      DebouncedEvent::Create(_) => c(&path, &sender),
      DebouncedEvent::Write(_) => c(&path, &sender),
      DebouncedEvent::Rename(_,_) => c(&path, &sender),
      DebouncedEvent::Rescan => c(&path, &sender),
      _ => (),
    }
  }
  
  let compile_editor = editor.clone();
  let (compile_sender, compile_receiver):(Sender<()>, Receiver<()>) = channel();
  thread::spawn(move || {
    loop {
      match watch_receiver.recv() {
          Ok(event) => {
            println!("{:?}", event);
            match (*compile_editor.read().unwrap()).current_page {
              Page::MainPage => (),
              Page::ProjectPage {ref path, ..} => {
                compile(path, &event, &compile_sender);
              },
              Page::EditPage{ref path, ..} => {
                compile(path, &event, &compile_sender);
              },
            };
          },
          Err(e) => println!("watch error: {:?}", e),
      };
    }
  });

  let webview = web_view::builder()
    .title("Sabines Webseiten-Editor")
    .content(Content::Url("http://localhost:54321"))
    .size(1024, 600)
    .resizable(true)
    .debug(true)
    .user_data(())
    .invoke_handler(|webview, arg| {
      println!("rpc: {:?}", arg);
      editor.write().unwrap().handle_external(webview, arg);
      Ok(())
    })
    .build()
    .unwrap();
    
  let css_injector_editor = editor.clone();
  let css_injector_webview = webview.handle();
  thread::spawn(move || {
    loop {
      match compile_receiver.recv() {
          Ok(()) => {
            match (*css_injector_editor.read().unwrap()).current_page {
              Page::MainPage => (),
              Page::ProjectPage{..} => (),
              Page::EditPage {..} => {
                css_injector_webview.dispatch(move |webview| {
                    (*webview).eval("window.reloadCss()");
                    Ok(())
                }).unwrap();
              },
            };
          },
          Err(e) => println!("watch error: {:?}", e),
      };
    }
  });
  
  
  let preview_editor = editor.clone();
  thread::spawn(move || {
    rouille::start_server("localhost:54322", move |request| {
      match get_path_from_editor(&*preview_editor.read().unwrap()) {
        Some(p) => {
          let response = rouille::match_assets(&request, &output_path(&p));
          if response.is_success() {
            return response;
          }
          println!("trying index.html...");
          let mut r = rouille::Request::fake_http("GET", request.url() + "/index.html", vec![], vec![]);
          rouille::match_assets(&r, &output_path(&p))
        },
        None => rouille::Response::empty_404(),
      }
    });
  });
  
  webview.run().unwrap();
}