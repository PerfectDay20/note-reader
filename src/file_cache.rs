use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct FilesCache {
    files: Vec<PathBuf>,
}

impl FilesCache {
    pub fn new(path: &Path) -> Self {
        if path.is_file() {
            FilesCache { files: vec![path.to_path_buf()] }
        } else {
            let mut files = vec![];
            Self::collect(path, &mut files);
            FilesCache { files }
        }
    }

    fn collect(p: &Path, v: &mut Vec<PathBuf>) {
        for e in p.read_dir().expect("read dir fail").flatten() {
            let p = e.path();
            if p.is_file() {
                if p.extension().unwrap_or_default() == "md" {
                    v.push(p)
                }
            } else {
                Self::collect(&p, v)
            }
        }
    }

    pub fn get_random(&self) -> Option<&PathBuf> {
        use rand::seq::IteratorRandom;
        let mut rng = rand::thread_rng();
        self.files.iter().choose(&mut rng)
    }
}
