> [!CAUTION]
> This library is under development. Documentation, functionality, and implementation details are not final. **Do NOT use in production.**

## Overview

```rust
use pslib::{ Document, Page, Rect, Line }
use std::{path::Path, fs::OpenOptions, io::BufWriter};

pub fn main() {

    // Prepare the output (boilerplate)
    let path = Path::new("output.ps");
    let file = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open(path)?;
    let mut writer = BufWriter::new(&file);

    // Create a new PostScript document (adds PostScript boilerplate)
    let doc = Document::new(writer);

    // Create a page: (w, h)
    let page = Page::new(400, 400);

    // Create a rectangle: (x, y, w, h)
    let rect = Rect::new(0, 0, 100, 100)
                    .fill_rgb(1.0, 0.0, 0.0);

    // Add rectangle to page (generates PS & writes to internal page buffer)
    page.add(&rect);

    // Create a line: (x1, y1, x2, y2)
    let line = Line::new(0, 100, 200, 100)
                    .stroke_rgb(1, 0.0, 0.0, 0.1);

    // Anything that impls the Serialize trait can be added to the page
    page.add(&line);

    // Add page to document (flushes internal page buffer to BufWriter)
    // Anything that impls Fabricate trait can be added to the document
    doc.add(&page);

    // Appends EOF
    doc.close();
}
```

## Serialize Trait

Anything that implements the `Serialize` trait can be added to a `Page` struct instance. The `Serialize` trait is used to convert a data structure to a multi-line PostScript string.

```rust
pub trait Serialize {
    fn to_postscript_string(&self) -> String;
}
```

### Example

```rust
struct Rect {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    strokeWidth: u8,
    strokeColor: [3; f32],
    fillColor: [3; f32],
}

impl Serialize for Rect {
    fn to_postscript_string(&self) -> String {
        let mut result = String::new();
        
        result.push_str("newpath\n");
        
        write!(&mut result, "{} {} moveto\n", self.x, self.y).unwrap();
        
        write!(&mut result, "0 {} rlineto\n", self.height).unwrap();
        write!(&mut result, "{} 0 rlineto\n", self.width).unwrap();
        write!(&mut result, "0 -{} rlineto\n", self.height).unwrap();
        write!(&mut result, "-{} 0 rlineto\n", self.width).unwrap();
        
        // Add the closepath and stroke commands
        result.push_str("closepath\n");

        if self.strokeWidth > 0 {
            write!(&mut result, "{} setlinewidth\n", self.strokeWidth).unwrap();
            write!(&mut result, "{} {} {} setrgbcolor\n", self.strokeColor[0], self.strokeColor[1], self.strokeColor[2]).unwrap();
            result.push_str("stroke\n");
        }
        
        result
    }
}
```

## Fabricate Trait

Anything that implements the `Fabricate` trait can be added to a `Document`. The `Fabricate` trait is used when you need to merge buffers (eg: writing a `Page` to a PostScript file).

```rust
pub trait Fabricate {
    fn fabricate<W: Write>(&self, doc_type: &DocumentType, writer: &mut BufWriter<W>) -> Result<(), Error>;
}
```

### Example

Appending a `Page` onto a `Document`.

```rust
struct Page {
    width: i32,
    height: i32,
    buffer: Vec<u8>,
}

impl Fabricate for Page {
    fn fabricate<W: Write>(&self, doc_type: &DocumentType, writer: &mut BufWriter<W>) -> Result<(), Error> {
        match doc_type {
            DocumentType::PS => {
                write!(
                    writer,
                    r#"
                        %%PageBoundingBox: 0 0 {} {}
                        << /PageSize [{} {}] >> setpagedevice
                    "#,
                    self.width, self.height, self.width, self.height
                )?;
                writer.write_all(&self.buffer)?;
                writer.write_all("showpage\n".as_bytes())?;
            }
            _ => {
                writer.write_all(&self.buffer)?;
            }
        }
        Ok(())
    }
}
```

## Document

Documents support writing to any type of buffer that implements the `Write` trait. Common usage includes:

- `File`
- `Vec<u8>`
- `stdout`
- `TcpStream`

When using `Document::new()` the document will be initialized with the default PostScript procedures defined by this library. If you need to define custom procedures see the document builder pattern section below.

```rust
use pslib::{ Document, Page };

fn main() {
    let path = Path::new("output.ps");
    let file = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open(path)?;
    let mut writer = BufWriter::new(&file);
    let doc = Document::new(writer);
    let page = Page::new(400, 400);
    doc.add(&page);
}
```

### Document Types

This library supports creating both PostScript and Encapsulated PostScript. Documents will default to PostScript when using `Document::new()`

```rust
enum DocumentType {
    PS, // PostScript
    EPS, // Encapsulated PostScript
}
```

### Builder

When creating a `Document` you can use the builder pattern.

```rust
use pslib::DocumentBuilder;

fn main() {
    let doc = Document::builder().build();
}
```

#### Setting the documents type

The `document_type()` method allows you to set a specific document type.

```rust
let doc = Document::builder().document_type(DocumentType::EPS).build();
```

#### Setting the documents buffer writer

When using the builder pattern the default the `BufWriter` will write to a `Vec<u8>` buffer. The `writer()` method allows you to set a specific buffer writer.

```rust
use pslib::{ DocumentBuilder, ProcedureRegistry };

fn main() {
    let path = Path::new("output.ps");
    let file = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open(path)?;
    let mut doc = DocumentBuilder::builder()
            .document_type(DocumentType::EPS)
            .writer(BufWriter::new(&file))
            .load_procedures(ProcedureRegistry::with_builtins())
            .bounding_box(500, 300)
            .build();
}
```

#### Loading procedures

The `load_procedures()` method allows you to initialize the document with a set of prebuilt PostScript procedures using the `ProcedureRegistry`.

```rust
let doc = DocumentBuilder::builder().load_procedures(ProcedureRegistry::with_builtins()).build();
```

## Procedures

PostScript allows us to define procedures that it pushes onto the operand stack (see [PLRM page 32-33](https://www.adobe.com/jp/print/postscript/pdfs/PLRM.pdf). These procedures can be repeatably executed to perform a predefine set of operations. 

Utilizing procedures increases interpreter performance while also reducing the overall final file size.

### Builtin Procedures

```rust
let registry = ProcedureRegistry::with_builtins();
let proc_rect = registry.get_procedure("rect").unwrap();
println!("{}", proc_rect);
```

```postscript
% ex: -100 0 0 -100 100 0 0 100 300 300 rect
/rect {
    newpath
    moveto
    rlineto
    rlineto
    rlineto
    rlineto
    closepath
} def
```

### Adding Custom Procedures

```rust
use pslib::{ ProcedureRegistry, Procedure };

fn main() {
    let registry = ProcedureRegistry::new();
    registry.add_procedure(Procedure {
        name: "custom_shape".to_string(),
        body: r#"
            /custom_shape {
                % Custom PostScript code
            } def
        "#.to_string(),
    });
}
```
## Line

```rust
use pslib::{ Line, TransformLineOrigin };

fn main() {
    let line = Line::new(100.0, 100.0, 100.0)
        .rotate(45.0)
        .set_orign(TransformLineOrigin::Left)
        .stroke_cmyk(2.0, 1.0, 0.0, 0.0, 0.25);
}
```

| Method | Parameters |
| - | - |
| `stroke_rgb` | `(width: f32, r: f32, g: f32, b: f32)` |
| `stroke_cmyk` | `(width: f32, c: f32, m: f32, y: f32, k: f32)` |
| `scale` | `(x: f32, y: f32)` |
| `set_orign` | `(origin: TransformOrigin)` |
| `rotate` | `(angle: f32)` |

## Rect

```rust
use pslib::Rect;

fn main() {
    let rect = Rect::new(155.0, 155.0, 100.0, 100.0)
        .fill_rgb(1.0, 0.0, 0.0)
        .rotate(45.0)
        .scale(1.5, 1.0)
        .stroke_rgb(2.0, 0.0, 0.0, 0.0);
}
```

| Method | Parameters |
| - | - |
| `fill_rgb` | `(r: f32, g: f32, b: f32)` |
| `fill_cmyk` | `(c: f32, m: f32, y: f32, k: f32)` |
| `stroke_rgb` | `(width: f32, r: f32, g: f32, b: f32)` |
| `stroke_cmyk` | `(width: f32, c: f32, m: f32, y: f32, k: f32)` |
| `scale` | `(x: f32, y: f32)` |
| `set_orign` | `(origin: TransformOrigin)` |
| `rotate` | `(angle: f32)` |

## Text

> [!WARNING]
> Text structure and implemention pending.

TODO:
- Alignment
    - Vertical
    - Horizontal
- Fit
    - Contain
    - Stretch
    - Crop

```rust
struct Text {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    stroke_width: f32,
    stroke_color_rgb: [f32; 3],
    stroke_color_cmyk: [f32; 4],
    fill_color_rgb: [f32; 3],
    fill_color_cmyk: [f32; 4],
    text: String,
    rotate: f32,
    scale: [f32; 2],
}
```

## Custom Fonts

Details pending.

## Image

Details pending.
