pub fn upload (host: &str, user: &str, pass: &str, remote: &str, source_path: &std::path::Path) -> () {
  use ftp::FtpStream;
  use std::io::Read;

  let mut ftp_stream = FtpStream::connect(host).unwrap();
  ftp_stream.login(user, pass).unwrap();
  println!("ls: {}", ftp_stream.list(None).unwrap().join("\n"));

  // TODO: this won't work, we need to remove files individually?
  //ftp_stream.rmdir(remote);

  for entry in walkdir::WalkDir::new(&source_path) {
    let e = entry.unwrap();
    if e.file_type().is_dir() {
      let d = e.path().strip_prefix(&source_path).unwrap();
      println!("dir: {:?}", d);
      
      for a in d.components() {
        let s = a.as_os_str().to_str().unwrap().replace("\\","/");
        ftp_stream.mkdir(&s);
        //println!("ls: {}", ftp_stream.list(None).unwrap().join("\n"));
        ftp_stream.cwd(&s);
      }
      ftp_stream.cwd(remote);
    }
    if e.file_type().is_file() {
      let mut source = std::fs::File::open(e.path()).unwrap();
      println!("source: {:?}", source);
      
      let target = e.path().strip_prefix(&source_path).unwrap();
      
      let target_path = target.to_str().unwrap().replace("\\","/");
      let info_path = target.parent().unwrap().join("__ftpinfo__".to_owned() + target.file_name().unwrap().to_str().unwrap()).to_str().unwrap().replace("\\","/");
      
      println!("target_path: {:?}", target_path);
      //println!("info_path: {:?}", info_path);
      
      let start = source.metadata().unwrap().modified().unwrap();
      let since_the_epoch = start.duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap();
      let mdtm = since_the_epoch.as_secs().to_string();
      
      let modified = match ftp_stream.simple_retr(&info_path) {
        Ok(mut info) => {
          let mut mdtm_upstream = String::new(); 
          info.read_to_string(&mut mdtm_upstream);
          println!("mdtm upstream/local: {:?}/{:?}", mdtm_upstream, mdtm);
          !std::string::String::eq(&mdtm, &mdtm_upstream)
        },
        Err(e) => {
          //println!("Error fetching mdtm: {:?}", e);
          true
        }
      };
      println!("modified: {:?} at {:?}", modified, mdtm);
      
      if modified {
        ftp_stream.put(&target_path, &mut source);
        ftp_stream.put(&info_path, &mut mdtm.as_bytes());
      }
    }
  }

  let _ = ftp_stream.quit();
}