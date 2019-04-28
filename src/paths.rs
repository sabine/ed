pub fn asset_path (path: &std::path::Path) -> std::path::PathBuf {
  path.join("assets/")
}

pub fn output_path (path: &std::path::Path) -> std::path::PathBuf {
  path.join("rust-www/")
}

pub fn pages_path (path: &std::path::Path) -> std::path::PathBuf {
  path.join("pages/")
}

pub fn data_path (path: &std::path::Path) -> std::path::PathBuf {
  path.join("data/")
}