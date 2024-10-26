use crate::{Serialize, TransformOrigin};
use std::fmt::Write;

pub struct Rect {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    stroke_width: u8,
    stroke_color: [f32; 3],
    fill_color: [f32; 3],
    do_fill: bool,
    rotate: f32,
    scale: [f32; 2],
    do_scale: bool,
    transform_origin: TransformOrigin,
}

impl Rect {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Rect {
            x: x.max(0),
            y: y.max(0),
            width: width.max(0),
            height: height.max(0),
            stroke_width: 0,
            stroke_color: [0.0, 0.0, 0.0],
            fill_color: [0.0, 0.0, 0.0],
            do_fill: false,
            rotate: 0.0,
            scale: [1.0, 1.0],
            do_scale: false,
            transform_origin: TransformOrigin::Center,
        }
    }

    pub fn set_fill(mut self, r: f32, g: f32, b: f32) -> Self {
        self.fill_color[0] = r.clamp(0.0, 1.0);
        self.fill_color[1] = g.clamp(0.0, 1.0);
        self.fill_color[2] = b.clamp(0.0, 1.0);
        self.do_fill = true;
        self
    }

    pub fn set_stroke(mut self, width: u8, r: f32, g: f32, b: f32) -> Self {
        self.stroke_width = width.max(0);
        self.stroke_color[0] = r.clamp(0.0, 1.0);
        self.stroke_color[1] = g.clamp(0.0, 1.0);
        self.stroke_color[2] = b.clamp(0.0, 1.0);
        self
    }

    pub fn set_scale(mut self, x: f32, y: f32) -> Self {
        self.scale[0] = x;
        self.scale[1] = y;
        self.do_scale = true;
        self
    }

    pub fn set_orign(mut self, origin: TransformOrigin) -> Self {
        self.transform_origin = origin;
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
            TransformOrigin::Center => (self.x + (self.width / 2), self.y + (self.height / 2)),
        };
        write!(&mut result, "{} {} translate\n", origin.0, origin.1).unwrap();

        if self.rotate > 0.0 {
            write!(
                &mut result,
                "{} {} transform\n",
                self.x + (self.width / 2),
                self.y + (self.height / 2)
            )
            .unwrap();
            write!(&mut result, "{} rotate\n", self.rotate).unwrap();
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

        if self.stroke_width > 0 {
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
