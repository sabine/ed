use paths;

#[derive(Debug)]
#[derive(Clone)]
pub struct Thumbnail {
  pub src: std::path::PathBuf,
  pub width: u32,
  pub height: u32,
  pub url: String,
}

impl std::fmt::Display for Thumbnail {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "< src: {:?}, ", self.src)?;
        write!(f, "width/height: {}/{}, url: {:?} > ", &self.width, &self.height, &self.url)?;
        Ok(())
    }
}

pub fn render_thumbnail (path: &std::path::Path, thumbnail: &Thumbnail) -> () {
  let p = paths::output_path(&path).join(&std::path::Path::new(&(".".to_string()+&thumbnail.url)));
  println!("render_thumbnail: {:?} -> {:?}\npath: {:?}", thumbnail, p, path);
  
  std::fs::create_dir_all(p.parent().unwrap());
  
  let image_path = &path.join(&thumbnail.src);
  println!("loading image: {:?}", image_path);
  
  // TODO: spit out a file stream thing instead of saving to file - so that this can be served by the internal web server
  
  match image::open(image_path) {
    Ok(img) => {
      img.thumbnail(thumbnail.width, thumbnail.height).save(&p);
    },
    Err(e) => {
      println!("{:?}", e);
    },
  };
}