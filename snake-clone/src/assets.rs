use engine_core::AssetSource;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "."]
struct GameAssets;

pub struct EmbeddedAssets;

impl AssetSource for EmbeddedAssets {
    fn read(&self, path: &str) -> Option<Vec<u8>> {
        GameAssets::get(path).map(|f| f.data.into_owned())
    }
}
