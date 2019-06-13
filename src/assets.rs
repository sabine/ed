use paths;


#[derive(Debug)]
#[derive(Clone)]
pub struct Asset {
  pub src: std::path::PathBuf,
  pub url: String,
}

impl std::fmt::Display for Asset {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "< src: {:?}, url: {:?} >", &self.src, &self.url)?;
        Ok(())
    }
}

pub fn collect_asset (path: &std::path::Path, asset: &Asset) -> () {
  let p = paths::output_path(&path).join(&std::path::Path::new(&(".".to_string()+&asset.url)));
  println!("collect_asset: {:?} -> {:?}\npath: {:?}", asset, p, path);
  
  std::fs::create_dir_all(p.parent().unwrap());

  std::fs::copy(&asset.src, &p);
}