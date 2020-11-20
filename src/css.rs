use std::error::Error;
use std::io::Write;

pub fn compile_sass(path: &std::path::Path, output_path: &std::path::Path) -> String {
  let mut cs = std::process::Command::new("./assets/sassc.exe");
  cs.arg(path.join("css/main.scss"));
  
  let r = cs.output().expect("SASS failed to execute").stdout;
  
  std::fs::create_dir_all(&output_path);
  let path = &output_path.join("main.css");
  let display = path.display();
  
  let mut file = match std::fs::File::create(&path) {
    Err(why) => panic!("couldn't create {}: {}",
                       display,
                       why.to_string()),
    Ok(file) => file,
  };

  match file.write_all(&r) {
    Err(why) => {
        panic!("couldn't write to {}: {}", display, why.to_string())
    },
    Ok(_) => println!("successfully wrote to {}", display),
  };
  String::from_utf8_lossy(&r).to_string()
}