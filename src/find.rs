use std::path::Path;

pub fn find_dirs(path_str: &str, depth: i32, mindepth: i32, maxdepth: i32) -> Vec<String> {
    println!("Searching in {:?} with depth {}", path_str, depth);
    if depth > maxdepth {
        return vec![];
    }
    let mut returnvector = vec![];
    let path = Path::new(path_str);
    if let Ok(dirs) = path.read_dir() {
        //println!("Directories in {:?}:", dirs);
        for entry in dirs {
            let Ok(entry) = entry else { continue };
            if entry.file_type().unwrap().is_dir() {
                if depth > mindepth {
                    returnvector.push(entry.path().display().to_string());
                }
                if depth >= maxdepth {
                    continue;
                }
                returnvector.append(&mut find_dirs(
                    entry.path().display().to_string().as_str(),
                    depth + 1,
                    mindepth,
                    maxdepth,
                ));
                //println!("{:?} {:?}", entry.path(), entry.file_type());
                //enter the Directory
            }
        }
    }
    returnvector
}
