pub fn upload (host: &str, user: &str, pass: &str, remote: &str, source_path: &std::path::Path) -> () {
  use ftp::FtpStream;

  let mut ftp_stream = FtpStream::connect(host).unwrap();
  ftp_stream.login(user, pass).unwrap();
  println!("ls: {}", ftp_stream.list(None).unwrap().join("\n"));

  ftp_stream.rmdir("/");

  for entry in walkdir::WalkDir::new(&source_path) {
    let e = entry.unwrap();
    if e.file_type().is_dir() {
      let d = e.path().strip_prefix(&source_path).unwrap();
      println!("dir: {:?}", d);
      
      for a in d.components() {
        let s = a.as_os_str().to_str().unwrap().replace("\\","/");
        ftp_stream.mkdir(&s);
        println!("ls: {}", ftp_stream.list(None).unwrap().join("\n"));
        ftp_stream.cwd(&s);
      }
      ftp_stream.cwd(remote);
    }
    if e.file_type().is_file() {
      let mut source = std::fs::File::open(e.path()).unwrap();
      let target = e.path().strip_prefix(&source_path).unwrap();
      println!("source: {:?}", source);
      println!("target: {:?}", target);
      ftp_stream.put(&target.to_str().unwrap().replace("\\","/"), &mut source);
    }
  }

  let _ = ftp_stream.quit();
}