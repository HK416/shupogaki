use std::io::Cursor;

use bevy::{
    asset::{AssetLoader, LoadContext, RenderAssetUsages, io::Reader},
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    tasks::ConditionalSendFuture,
};

/// An error that can occur when loading a texel data.
#[derive(Debug, thiserror::Error)]
pub enum TexelLoaderError {
    /// An I/O error occurred.
    #[error("Failed to load asset for the following reason:{0}")]
    IO(#[from] std::io::Error),
    /// A JSON deserialization error occurred.
    #[error("Failed to decode asset for the following reason:{0}")]
    Json(#[from] serde_json::Error),
    #[error("Failed to decode asset for the following reason:{0}")]
    Decode(#[from] image::ImageError),
}

/// A loader for texel assets.
#[derive(Default)]
pub struct TexelAssetLoader;

impl AssetLoader for TexelAssetLoader {
    type Asset = Image;
    type Settings = ();
    type Error = TexelLoaderError;

    fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext,
    ) -> impl ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        info!("asset load: {}", &load_context.asset_path());
        Box::pin(async move {
            // Read the bytes from the reader.
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;

            // TODO: 데이터 복호화
            // TODO: 데이터 압축 해제

            // Decode the image data using the `image` crate and create a Bevy `Image` asset.
            let mut reader = image::ImageReader::new(Cursor::new(bytes));
            reader.set_format(image::ImageFormat::Png);

            let image = reader.decode()?;
            let size = Extent3d {
                width: image.width(),
                height: image.height(),
                depth_or_array_layers: 1,
            };
            let dimension = TextureDimension::D2;
            let data = image.to_rgba8().to_vec();
            let format = TextureFormat::Rgba8UnormSrgb;
            let asset_usage = RenderAssetUsages::RENDER_WORLD;

            Ok(Image::new(size, dimension, data, format, asset_usage))
        })
    }

    fn extensions(&self) -> &[&str] {
        &["texture"]
    }
}
