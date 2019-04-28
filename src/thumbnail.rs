use paths;

#[derive(Debug)]
#[derive(Clone)]
pub struct ThumbnailRequest {
  pub src: std::path::PathBuf,
  pub width: u32,
  pub height: u32,
  pub url: String,
}

impl std::fmt::Display for ThumbnailRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "< src: {:?}, ", self.src)?;
        write!(f, "width/height: {}/{} > ", &self.width, &self.height)?;
        Ok(())
    }
}

pub fn render_thumbnail (path: &std::path::Path, thumbnail_request: &ThumbnailRequest) -> () {
  let p = paths::output_path(&path).join(&std::path::Path::new(&(".".to_string()+&thumbnail_request.url)));
  println!("render_thumbnail: {:?} -> {:?}", thumbnail_request, p);
  
  std::fs::create_dir_all(p.parent().unwrap());
  
  let img = image::open(&paths::asset_path(&path).join(&thumbnail_request.src)).unwrap();
  img.thumbnail(thumbnail_request.width, thumbnail_request.height).save(&p);
}