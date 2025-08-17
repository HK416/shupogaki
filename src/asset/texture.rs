use bevy::{
    asset::{Asset, AssetLoader, LoadContext, io::Reader},
    reflect::TypePath,
    tasks::ConditionalSendFuture,
};

/// A custom asset for textures that can be processed (e.g., decrypted, decompressed)
/// before being used by the renderer.
#[derive(Asset, TypePath)]
pub struct TexelAsset {
    pub data: Vec<u8>,
}

/// An error that can occur when loading a texel data.
#[derive(Debug, thiserror::Error)]
pub enum TexelLoaderError {
    /// An I/O error occurred.
    #[error("Failed to load asset for the following reason:{0}")]
    IO(#[from] std::io::Error),
    /// A JSON deserialization error occurred.
    #[error("Failed to decode asset for the following reason:{0}")]
    Json(#[from] serde_json::Error),
}

/// A loader for texel assets.
#[derive(Default)]
pub struct TexelAssetLoader;

impl AssetLoader for TexelAssetLoader {
    type Asset = TexelAsset;
    type Settings = ();
    type Error = TexelLoaderError;

    fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext,
    ) -> impl ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            // Read the bytes from the reader.
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;

            // TODO: 데이터 복호화
            // TODO: 데이터 압축 해제
            // Create the `TexelAsset`
            Ok(TexelAsset { data: bytes })
        })
    }

    fn extensions(&self) -> &[&str] {
        &["tex"]
    }
}
