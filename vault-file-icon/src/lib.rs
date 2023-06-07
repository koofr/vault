mod file_icon;
#[cfg(feature = "render")]
mod render;

pub use file_icon::{
    FileIconAttrs, FileIconCategory, FileIconFactory, FileIconProps, FileIconSize, FileIconTheme,
};

#[cfg(feature = "render")]
pub use render::{render_png, RenderPngError};
