use crate::{Serialize, TransformOrigin};
use std::fmt::Write;

pub struct Rect {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    stroke_width: f32,
    stroke_color: [f32; 3],
    fill_color: [f32; 3],
    do_fill: bool,
    rotate: f32,
    scale: [f32; 2],
    do_scale: bool,
    transform_origin: TransformOrigin,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Rect {
            x: x.max(0.0),
            y: y.max(0.0),
            width: width.max(0.0),
            height: height.max(0.0),
            stroke_width: 0.0,
            stroke_color: [0.0, 0.0, 0.0],
            fill_color: [0.0, 0.0, 0.0],
            do_fill: false,
            rotate: 0.0,
            scale: [1.0, 1.0],
            do_scale: false,
            transform_origin: TransformOrigin::Center,
        }
    }

    pub fn fill(mut self, r: f32, g: f32, b: f32) -> Self {
        self.fill_color[0] = r.clamp(0.0, 1.0);
        self.fill_color[1] = g.clamp(0.0, 1.0);
        self.fill_color[2] = b.clamp(0.0, 1.0);
        self.do_fill = true;
        self
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

    pub fn set_orign(mut self, origin: TransformOrigin) -> Self {
        self.transform_origin = origin;
        self
    }

    pub fn rotate(mut self, angle: f32) -> Self {
        self.rotate = angle.clamp(0.0, 360.0);
        self
    }
}

impl Serialize for Rect {
    fn to_postscript_string(&self) -> String {
        let mut result = String::new();

        result.push_str("gsave\n");

        let origin = match self.transform_origin {
            TransformOrigin::TopLeft => (self.x, self.y + self.height),
            TransformOrigin::TopRight => (self.x + self.width, self.y + self.height),
            TransformOrigin::BottomLeft => (self.x, self.y),
            TransformOrigin::BottomRight => (self.x + self.width, self.y),
            TransformOrigin::Center => (self.x + (self.width / 2.0), self.y + (self.height / 2.0)),
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
            "-{} 0 0 -{} {} 0 0 {} {} {} rect\n",
            self.width, self.height, self.width, self.height, self.x, self.y
        )
        .unwrap();

        if self.do_fill {
            write!(&mut result, "{} setlinewidth\n", self.stroke_width).unwrap();
            write!(
                &mut result,
                "{} {} {} setrgbcolor\n",
                self.fill_color[0], self.fill_color[1], self.fill_color[2]
            )
            .unwrap();
            result.push_str("gsave\n");
            result.push_str("fill\n");
            result.push_str("grestore\n");
        }

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
