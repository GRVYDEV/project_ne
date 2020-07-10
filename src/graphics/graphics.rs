use luminance::pixel::NormRGBA8UI;
use luminance::texture::Dim2;
use luminance::context::GraphicsContext;
use luminance::texture::Texture as lum_Texture;
pub fn load_texture_from_bytes<C>(context: &mut C, bytes: &[u8]) -> lum_Texture<Dim2, NormRGBA8UI>
where
    C: GraphicsContext,
{
    use image::GenericImageView;
    use luminance::texture::Sampler;
    use luminance::texture::{MagFilter, MinFilter, Wrap};
    let image =
        image::DynamicImage::ImageRgba8(image::load_from_memory(bytes).unwrap().into_rgba());

    let mut texture = lum_Texture::new(
        context,
        [image.width(), image.height()],
        0,
        Sampler {
            wrap_r: Wrap::ClampToEdge,
            wrap_s: Wrap::ClampToEdge,
            wrap_t: Wrap::ClampToEdge,
            min_filter: MinFilter::Nearest,
            mag_filter: MagFilter::Nearest,
            depth_comparison: None,
        },
    )
    .unwrap();
    texture.upload_raw(luminance::texture::GenMipmaps::No, &image.to_bytes());
    texture
}