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
    horizontal_align: TextHorizontalAlignment,
    vertical_align: TextVerticalAlignment,
    fit: TextFit,
    wrap: TextWrap,
}

enum TextFit {
    Contain, // default
    Stretch,
    StretchHorizontal,
    StretchVertical,
    Crop,
}

enum TextHorizontalAlignment {
    Left, // default
    Center,
    Right,
}

enum TextVerticalAlignment {
    Top, // default
    Center,
    Bottom,
}

enum TextWrap {
    Wrap,
    Nowrap, // default
}
```

### Example

```rust
let text = Text::new("Hello, World!", 0.0, 0.0, 100.0, 50.0)
                    .horizontal_align(TextHorizontalAlignment::Center)
                    .vertical_align(TextVerticalAlignment::Center)
                    .fit(TextFit::StretchHorizontal)
                    .wrap(TextWrap::Wrap);
```

## Custom Fonts

Details pending.

## Image

> [!WARNING]
> Image structure and implemention pending.

I'm thinking I'd like images to be automatically conveted to PostScript procedures. That way if/when we draw the same image to the document several times we don't have to binary encode the image every time.

My first instinct is to create another registry like the `ProcedureRegistry` but this would force developers to register all the images they're going to use up front.

Experiment needed: test to see if the code that invokes a procedure can come before the definition of the procedure.

```rust
struct Image {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    rotate: f32,
    scale: [f32; 2],
    procedure_id: String,
}

impl Image {
    pub fn new(procedure_id: String, x: f32, y: f32, width: f32, height: f32) -> Self {
        Image {
            x: x.max(0.0),
            y: y.max(0.0),
            width: width.max(0.0),
            height: height.max(0.0),
            rotate: 0.0,
            scale: [0.0, 0.0],
            procedure_id: procedure_id,
        }
    }
}
```

### Image Registry

```rust
struct RawImage {
    file_name: String,
    file_path: Path,
    procedure_name: String,
}
struct ImageRegistry {
    images: HashMap<String, RawImage>,
    count: u32,
}

impl ImageRegistry {
    pub fn new() -> Self {
        ImageRegistry {
            images: HashMap::new(),
            count: 0,
        }
    }

    pub fn add(mut self, path: Path) -> Self {
        let file_name = path.file_name().to_string_lossy();
        self.count += 1;
        let proc_name = format!("imager{}", self.count);
        let image = RawImage {
            file_name: file_name,
            file_path: path,
            procedure_name: proc_name,
        };
        self.images.add(file_name, image);
        self
    }

    pub fn get_procedure_id(self, file_name: String) -> Option<String> {
        let raw = self.images.get(file_name);
        if raw.is_none() {
            return None;
        }
        Some(raw.procedure_name)
    }
}
```

### Example

```rust
let image_path = Path::new("/some/directory/filename.txt");
let registry = ProcedureRegistry::new();
registry.add(image_path);

let buffer: Vec<u8> = Vec::new();
let mut writer = BufWriter::new(&buffer);
let document = Document::new(writer);

document.load_images(registry); // generates and writes image procedures to buffer

let page = Page::new(100.0, 100.0);

let image = Image::new(registry.get_procedure_id("filename.txt", 0.0, 0.0, 100.0, 100.0));
page.add(&image); // writes "100 100 0 0 imager1" to buffer
```

## Inline Images

> [!WARNING]
> Inline image structure and implemention pending.

Unlike the standard `Image` that invokes a stored image procedure defiend by the `ImageRegistry` inline images will write the binary encoded image directy into the `Page` every time. This will most likely be useful when a developer _knows_ they will only write the image once. It may also be useful (even recommended?) when creating EPS files.

The final documentation should strongly encourage developers to use the `ImageRegistry`.

```rust
struct InlineImage {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    rotate: f32,
    scale: [f32; 2],
    file_path: Path,
}

impl InlineImage {
    pub fn new(file_path: Path, x: f32, y: f32, width: f32, height: f32) -> Self {
        InlineImage {
            x: x.max(0.0),
            y: y.max(0.0),
            width: width.max(0.0),
            height: height.max(0.0),
            rotate: 0.0,
            scale: [0.0, 0.0],
            file_path: file_path,
        }
    }
}
```