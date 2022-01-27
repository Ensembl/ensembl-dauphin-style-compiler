use std::{io, fs::{read_to_string, read, metadata}, hash::{Hash}, path::{Path}, os::unix::prelude::MetadataExt};

use crate::{core::error::AssemblerError, suite::assets::{AssetLoad, AssetLoader}};

pub struct FileLoader {
    search_paths: Vec<String>
}

impl FileLoader {
    pub fn new() -> FileLoader {
        FileLoader {
            search_paths: vec![]
        }
    }

    pub fn add_search_path(&mut self, path: &str) {
        self.search_paths.push(path.to_string());
    }

    fn find_file(&self, path: &str, context_path: &Option<String>, search: bool) -> Result<String,AssemblerError> {
        let mut tried = vec![];
        let rest = Path::new(path);
        let dot = vec![".".to_string()];
        let search_paths = if search { &self.search_paths } else { &dot };
        if search_paths.len() == 0 {
            return Err(AssemblerError::FileError(format!("no such path (no paths to try)")));
        }
        for root in search_paths {
            let root = if let Some(context_path) = context_path {
                let mut context_dir = Path::new(context_path).to_path_buf();
                context_dir.pop();
                context_dir.join(root).to_path_buf()
            } else {
                Path::new(root).to_path_buf()
            };
            let path = root.join(rest);
            tried.push(path.to_string_lossy().to_string());
            if path.exists() {
                return Ok(path.to_str().ok_or_else(|| AssemblerError::FileError("cannot convert path".to_string()))?.to_string());
            }
        }
        Err(AssemblerError::FileError(format!("no such path (tried {})",tried.join(", "))))
    }

    pub(crate) fn make_load_file(&self, path: &str, context_path: &Option<String>, search: bool) -> Result<InputFile,AssemblerError> {
        InputFile::find(&self,path,context_path,search)
    }
}

impl AssetLoader for FileLoader {
    fn make_load<'a>(&'a self, path: &str, context_path: &Option<String>, search: bool) -> Result<Box<dyn AssetLoad + 'a>,AssemblerError> {
        Ok(Box::new(self.make_load_file(path,context_path,search)?))
    }
}

#[derive(Eq)]
pub(crate) struct InputFile {
    id: (u64,u64),
    path: String,
}

impl PartialEq for InputFile {
    fn eq(&self, other: &Self) -> bool { self.id == other.id }
}

impl Hash for InputFile {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) { self.id.hash(state); }
}

impl InputFile {
    pub(crate) fn new(path: &str) -> Result<InputFile,AssemblerError> {
        let id = InputFile::io_error(&path,metadata(&path).map(|md| (md.dev(),md.ino())))?;
        Ok(InputFile {
            id,
            path: path.to_string(),
        })
    }

    fn find(loader: &FileLoader, path: &str, context_path: &Option<String>, search: bool) -> Result<InputFile,AssemblerError> {
        InputFile::new(&InputFile::add_error_path(path,loader.find_file(&path,&context_path,search))?)
    }

    pub(crate) fn file_path(&self) -> &str { &self.path }

    fn add_error_path<T>(path: &str, t: Result<T,AssemblerError>) -> Result<T,AssemblerError> {
        t.map_err(|e| {
            e.add_context(&format!("loading {}",path))
        })
    }

    fn io_error<T>(path: &str, t: io::Result<T>) -> Result<T,AssemblerError> {
        InputFile::add_error_path(path,t.map_err(|e| AssemblerError::FileError(e.to_string())))
    }
}

impl AssetLoad for InputFile {
    fn load_bytes(&self) -> Result<Vec<u8>,AssemblerError> {
        InputFile::io_error(&self.path,read(&self.path))
    }

    fn load_string(&self) -> Result<String,AssemblerError> {
        InputFile::io_error(&self.path,read_to_string(&self.path))
    }
}

#[cfg(test)]
mod test {
    use std::env::current_dir;

    use crate::core::testutil::no_error;

    use super::FileLoader;

    #[test]
    fn test_fileloader_smoke() {
        let mut loader = FileLoader::new();
        loader.add_search_path(".");
        let load = no_error(loader.find_file("src/test/assets/raw-asset.bin",&None,true));
        assert_eq!("./src/test/assets/raw-asset.bin",&load);

        let load = no_error(loader.find_file("assets/raw-asset.bin",&Some("src/test/x".to_string()),true));
        assert_eq!("src/test/./assets/raw-asset.bin",&load);

        let mut loader2 = FileLoader::new();
        loader2.add_search_path("test");
        let load = no_error(loader2.find_file("assets/raw-asset.bin",&Some("src/x".to_string()),true));
        assert_eq!("src/test/assets/raw-asset.bin",&load);
    }

    #[test]
    fn test_fileloader_abs_search() {
        let cwd = no_error(current_dir());
        let mut loader = FileLoader::new();
        loader.add_search_path(&cwd.to_string_lossy());
        let load = no_error(loader.find_file("src/test/assets/raw-asset.bin",&Some("src/test/x".to_string()),true));
        assert_eq!(&format!("{}/src/test/assets/raw-asset.bin",cwd.to_string_lossy()),&load);
    }

    #[test]
    fn test_fileloader_multiple() {
        let mut loader = FileLoader::new();
        loader.add_search_path("test1");
        loader.add_search_path("test2");
        let load = no_error(loader.find_file("test-file",&Some("src/test/assets/x".to_string()),true));
        assert_eq!("src/test/assets/test1/test-file",&load);
        let load = no_error(loader.find_file("test-file2",&Some("src/test/assets/x".to_string()),true));
        assert_eq!("src/test/assets/test2/test-file2",&load);

        let mut loader = FileLoader::new();
        loader.add_search_path("test2");
        loader.add_search_path("test1");
        let load = no_error(loader.find_file("test-file",&Some("src/test/assets/x".to_string()),true));
        assert_eq!("src/test/assets/test2/test-file",&load);
    }
}
