pub trait AssetSource {
    fn read(&self, path: &str) -> Option<Vec<u8>>;

    fn read_to_string(&self, path: &str) -> Option<String> {
        self.read(path).and_then(|bytes| String::from_utf8(bytes).ok())
    }
}

pub struct FilesystemAssets;

impl AssetSource for FilesystemAssets {
    fn read(&self, path: &str) -> Option<Vec<u8>> {
        std::fs::read(path).ok()
    }
    
}
