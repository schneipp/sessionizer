use std::path::Path;

pub fn find_dirs(path_str: &str, depth: i32) -> Vec<String> {
    if depth > 3 {
        return vec![];
    }
    //list direcotries in the path
    let mut returnvector = vec![];
    let path = Path::new(path_str);
    if let Ok(dirs) = path.read_dir() {
        //println!("Directories in {:?}:", dirs);
        for entry in dirs {
            let Ok(entry) = entry else { continue };
            if entry.file_type().unwrap().is_dir() {
                returnvector.push(entry.path().display().to_string());
                returnvector.append(&mut find_dirs(
                    entry.path().display().to_string().as_str(),
                    depth + 1,
                ));
                //println!("{:?} {:?}", entry.path(), entry.file_type());
                //enter the Directory
            }
        }
    }
    returnvector
}
