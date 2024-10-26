use crate::Serialize;
use std::fmt::Write;

pub enum TransformLineOrigin {
    Left,
    Center, // default
    Right,
}

pub struct Line {
    x: f32,
    y: f32,
    length: f32,
    stroke_width: f32,
    stroke_color: [f32; 3],
    rotate: f32,
    scale: [f32; 2],
    do_scale: bool,
    transform_origin: TransformLineOrigin,
}

impl Line {
    pub fn new(x: f32, y: f32, length: f32) -> Self {
        Line {
            x: x.max(0.0),
            y: y.max(0.0),
            length: length.max(0.0),
            stroke_width: 1,
            stroke_color: [0.0, 0.0, 0.0],
            rotate: 0.0,
            scale: [1.0, 1.0],
            do_scale: false,
            transform_origin: TransformLineOrigin::Center,
        }
    }

    pub fn stroke(mut self, width: f32, r: f32, g: f32, b: f32) -> Self {
        self.stroke_width = width.max(0.0);
        self.stroke_color[0] = r.clamp(0.0, 1.0);
        self.stroke_color[1] = g.clamp(0.0, 1.0);
        self.stroke_color[2] = b.clamp(0.0, 1.0);
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
        self.rotate = angle.clamp(0.0, 360.0);
        self
    }
}

impl Serialize for Line {
    fn to_postscript_string(&self) -> String {
        let mut result = String::new();

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
            write!(&mut result, "-{} rotate\n", self.rotate).unwrap();
        }

        if self.do_scale {
            write!(&mut result, "{} {} scale\n", self.scale[0], self.scale[1]).unwrap();
        }

        write!(
            &mut result,
            "{} {} {} {} line\n",
            self.x + self.length,
            self.y,
            self.x,
            self.y
        )
        .unwrap();

        if self.stroke_width > 0.0 {
            write!(&mut result, "{} setlinewidth\n", self.stroke_width).unwrap();
            write!(
                &mut result,
                "{} {} {} setrgbcolor\n",
                self.stroke_color[0], self.stroke_color[1], self.stroke_color[2]
            )
            .unwrap();
            result.push_str("gsave\n");
            result.push_str("stroke\n");
            result.push_str("grestore\n");
        }

        result.push_str("grestore\n");

        result
    }
}
