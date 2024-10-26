use crate::{ColorMode, Serialize, TransformLineOrigin};
use std::fmt::Write;

pub struct Line {
    x: f32,
    y: f32,
    length: f32,
    stroke_width: f32,
    stroke_color_rgb: [f32; 3],
    stroke_color_cmyk: [f32; 4],
    rotate: f32,
    scale: [f32; 2],
    do_scale: bool,
    do_rotate: bool,
    transform_origin: TransformLineOrigin,
    color_mode: ColorMode,
}

impl Line {
    pub fn new(x: f32, y: f32, length: f32) -> Self {
        Line {
            x: x.max(0.0),
            y: y.max(0.0),
            length: length.max(0.0),
            stroke_width: 1.0,
            stroke_color_rgb: [0.0, 0.0, 0.0],
            stroke_color_cmyk: [0.0, 0.0, 0.0, 0.0],
            rotate: 0.0,
            scale: [1.0, 1.0],
            do_scale: false,
            do_rotate: false,
            transform_origin: TransformLineOrigin::Center,
            color_mode: ColorMode::RGB,
        }
    }

    pub fn stroke_rgb(mut self, width: f32, r: f32, g: f32, b: f32) -> Self {
        self.stroke_width = width.max(0.0);
        self.stroke_color_rgb[0] = r.clamp(0.0, 1.0);
        self.stroke_color_rgb[1] = g.clamp(0.0, 1.0);
        self.stroke_color_rgb[2] = b.clamp(0.0, 1.0);
        self.color_mode = ColorMode::RGB;
        self
    }

    pub fn stroke_cmyk(mut self, width: f32, c: f32, m: f32, y: f32, k: f32) -> Self {
        self.stroke_width = width.max(0.0);
        self.stroke_color_cmyk[0] = c.clamp(0.0, 1.0);
        self.stroke_color_cmyk[1] = m.clamp(0.0, 1.0);
        self.stroke_color_cmyk[2] = y.clamp(0.0, 1.0);
        self.stroke_color_cmyk[3] = k.clamp(0.0, 1.0);
        self.color_mode = ColorMode::CMYK;
        self
    }

    pub fn scale(mut self, x: f32, y: f32) -> Self {
        self.scale[0] = x;
        self.scale[1] = y;
        self.do_scale = true;
        self
    }

    pub fn set_orign(mut self, origin: TransformLineOrigin) -> Self {
        self.transform_origin = origin;
        self
    }

    pub fn rotate(mut self, angle: f32) -> Self {
        self.rotate = angle.clamp(-360.0, 360.0);
        self.do_rotate = true;
        self
    }
}

impl Serialize for Line {
    fn to_postscript_string(&self) -> String {
        let mut result = String::new();

        if self.do_rotate || self.do_scale {
            result.push_str("gsave\n");
            let origin = match self.transform_origin {
                TransformLineOrigin::Left => (self.x, self.y + (self.stroke_width / 2.0)),
                TransformLineOrigin::Center => (
                    self.x + (self.length / 2.0),
                    self.y + (self.stroke_width / 2.0),
                ),
                TransformLineOrigin::Right => {
                    (self.x + self.length, self.y + (self.stroke_width / 2.0))
                }
            };
            write!(&mut result, "{} {} translate\n", origin.0, origin.1).unwrap();

            if self.rotate > 0.0 && self.rotate < 360.0 {
                write!(&mut result, "{} rotate\n", self.rotate).unwrap();
            }

            if self.do_scale {
                write!(&mut result, "{} {} scale\n", self.scale[0], self.scale[1]).unwrap();
            }

            write!(&mut result, "-{} -{} translate\n", origin.0, origin.1).unwrap();
        }

        write!(
            &mut result,
            "{} 0 {} {} line\n",
            self.length, self.x, self.y,
        )
        .unwrap();

        if self.stroke_width > 0.0 {
            result.push_str("gsave\n");
            write!(&mut result, "{} setlinewidth\n", self.stroke_width).unwrap();
            match self.color_mode {
                ColorMode::RGB => {
                    write!(
                        &mut result,
                        "{} {} {} setrgbcolor\n",
                        self.stroke_color_rgb[0],
                        self.stroke_color_rgb[1],
                        self.stroke_color_rgb[2]
                    )
                    .unwrap();
                }
                ColorMode::CMYK => {
                    write!(
                        &mut result,
                        "{} {} {} {} setcmykcolor\n",
                        self.stroke_color_cmyk[0],
                        self.stroke_color_cmyk[1],
                        self.stroke_color_cmyk[2],
                        self.stroke_color_cmyk[3],
                    )
                    .unwrap();
                }
            }
            result.push_str("stroke\n");
            result.push_str("grestore\n");
        }

        if self.do_rotate || self.do_scale {
            result.push_str("grestore\n");
        }

        result
    }
}
