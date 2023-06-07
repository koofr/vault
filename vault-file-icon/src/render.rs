use usvg::TreeParsing;

#[derive(Debug, Clone)]
pub struct RenderPngError(String);

impl std::fmt::Display for RenderPngError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for RenderPngError {}

pub fn render_png(svg: &str, width: u32, height: u32) -> Result<Vec<u8>, RenderPngError> {
    let opt = usvg::Options::default();

    let tree = usvg::Tree::from_data(svg.as_bytes(), &opt)
        .map_err(|err| RenderPngError(format!("failed to parse svg: {}", err)))?;

    let rtree = resvg::Tree::from_usvg(&tree);

    // fails on zero size
    let mut pixmap = tiny_skia::Pixmap::new(width, height).unwrap();

    let svg_size = tree.size.to_int_size().to_size();
    let png_size = tiny_skia::Size::from_wh(width as f32, height as f32).unwrap();

    let transform = tiny_skia::Transform::from_scale(
        png_size.width() / svg_size.width(),
        png_size.height() / svg_size.height(),
    );

    rtree.render(transform, &mut pixmap.as_mut());

    Ok(pixmap
        .encode_png()
        .map_err(|err| RenderPngError(format!("failed to encode png: {}", err)))?)
}
