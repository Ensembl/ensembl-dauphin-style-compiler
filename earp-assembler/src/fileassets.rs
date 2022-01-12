use std::{io, fs::{read_to_string, read}, path::{Path}};

use crate::{error::AssemblerError, assets::{AssetLoad, AssetLoader}};

pub(crate) struct FileAssetLoader {
    search_paths: Vec<String>
}

impl FileAssetLoader {
    pub(crate) fn new() -> FileAssetLoader {
        FileAssetLoader {
            search_paths: vec![]
        }
    }

    pub(crate) fn add_search_path(&mut self, path: &str) {
        self.search_paths.push(path.to_string());
    }

    fn find_file(&self, path: &str, context_path: &Option<String>) -> Result<String,AssemblerError> {
        let mut tried = vec![];
        let rest = Path::new(path);
        if self.search_paths.len() == 0 {
            return Err(AssemblerError::FileError(format!("no such path (no paths to try)")));
        }
        for root in &self.search_paths {
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
}

impl AssetLoader for FileAssetLoader {
    fn make_load<'a>(&'a self, path: &str, context_path: &Option<String>) -> Result<Box<dyn AssetLoad + 'a>,AssemblerError> {
        Ok(Box::new(FileAssetLoad::new(&self,path,context_path)?))
    }
}

pub(crate) struct FileAssetLoad<'a> {
    loader: &'a FileAssetLoader,
    path: String,
    context_path: Option<String>
}

impl<'a> FileAssetLoad<'a> {
    fn new(loader: &'a FileAssetLoader, path: &str, context_path: &Option<String>) -> Result<FileAssetLoad<'a>,AssemblerError> {
        Ok(FileAssetLoad {
            loader, path: path.to_string(),
            context_path: context_path.clone()
        })
    }

    fn add_error_path<T>(&self, t: Result<T,AssemblerError>) -> Result<T,AssemblerError> {
        t.map_err(|e| {
            e.add_context(&format!("loading asset {}",self.path))
        })
    }

    fn io_error<T>(&self, t: io::Result<T>) -> Result<T,AssemblerError> {
        self.add_error_path(t.map_err(|e| AssemblerError::FileError(e.to_string())))
    }
}

impl<'a> AssetLoad for FileAssetLoad<'a> {
    fn load_bytes(&self) -> Result<Vec<u8>,AssemblerError> {
        let path = self.add_error_path(self.loader.find_file(&self.path,&self.context_path))?;
        self.io_error(read(path))
    }

    fn load_string(&self) -> Result<String,AssemblerError> {
        let path = self.add_error_path(self.loader.find_file(&self.path,&self.context_path))?;
        self.io_error(read_to_string(path))
    }
}

#[cfg(test)]
mod test {
    use std::env::current_dir;

    use crate::testutil::no_error;

    use super::FileAssetLoader;

    #[test]
    fn test_fileloader_smoke() {
        let mut loader = FileAssetLoader::new();
        loader.add_search_path(".");
        let load = no_error(loader.find_file("src/test/assets/raw-asset.bin",&None));
        assert_eq!("./src/test/assets/raw-asset.bin",&load);

        let load = no_error(loader.find_file("assets/raw-asset.bin",&Some("src/test/x".to_string())));
        assert_eq!("src/test/./assets/raw-asset.bin",&load);

        let mut loader2 = FileAssetLoader::new();
        loader2.add_search_path("test");
        let load = no_error(loader2.find_file("assets/raw-asset.bin",&Some("src/x".to_string())));
        assert_eq!("src/test/assets/raw-asset.bin",&load);
    }

    #[test]
    fn test_fileloader_abs_search() {
        let cwd = no_error(current_dir());
        let mut loader = FileAssetLoader::new();
        loader.add_search_path(&cwd.to_string_lossy());
        let load = no_error(loader.find_file("src/test/assets/raw-asset.bin",&Some("src/test/x".to_string())));
        assert_eq!(&format!("{}/src/test/assets/raw-asset.bin",cwd.to_string_lossy()),&load);
    }

    #[test]
    fn test_fileloader_multiple() {
        let mut loader = FileAssetLoader::new();
        loader.add_search_path("test1");
        loader.add_search_path("test2");
        let load = no_error(loader.find_file("test-file",&Some("src/test/assets/x".to_string())));
        assert_eq!("src/test/assets/test1/test-file",&load);
        let load = no_error(loader.find_file("test-file2",&Some("src/test/assets/x".to_string())));
        assert_eq!("src/test/assets/test2/test-file2",&load);

        let mut loader = FileAssetLoader::new();
        loader.add_search_path("test2");
        loader.add_search_path("test1");
        let load = no_error(loader.find_file("test-file",&Some("src/test/assets/x".to_string())));
        assert_eq!("src/test/assets/test2/test-file",&load);
    }
}
