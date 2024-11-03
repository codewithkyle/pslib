use std::path::Path;

struct InlineImage {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    rotate: f32,
    scale: [f32; 2],
    file_path: Box<Path>,
    fit: ImageFit,
}

enum ImageFit {
    Contain,
    Stretch,
    StretchHorizontal,
    StretchVertical,
    Crop,
}

impl InlineImage {
    pub fn new(file_path: &Path, x: f32, y: f32, width: f32, height: f32) -> Self {
        InlineImage {
            x: x.max(0.0),
            y: y.max(0.0),
            width: width.max(0.0),
            height: height.max(0.0),
            rotate: 0.0,
            scale: [0.0, 0.0],
            file_path: file_path.into(),
            fit: ImageFit::Contain,
        }
    }
}
