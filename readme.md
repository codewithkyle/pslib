> **Note:** this library is under development. Documentation, functionality, and implementation details are not final.

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
    let rect = Rect::new(0, 0, 100, 100);

    // Set fill RGB color: (r, g, b)
    rect.setFill(1.0, 0.0, 0.0);

    // Add rectangle to page (generates PS & writes to internal page buffer)
    page.add(rect);

    // Create a line: (x1, y1, x2, y2)
    let line = Line::new(0, 100, 200, 100);

    // Set stroke size & color: (size, r, g, b)
    line.setStroke(1, 0.0, 0.0, 0.1);

    // Anything that impls the Serialize trait can be added to the page
    page.add(line);

    // Add page to document (flushes internal page buffer to BufWriter)
    // Anything that impls Fabricate trait can be added to the document
    doc.add(&page);

    // Appends EOF and closes BufWriter
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
    fn fabricate<W: Write>(&self, writer: &mut BufWriter<W>) -> Result<()>;
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
    fn fabricate<W: Write>(&self, writer: &mut BufWriter<W>) -> Result<()> {
        writer.write_all(&self.buffer)?;
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

```rust
struct Document<W: Write> {
    doc_type: DocumentType,
    buffer: BufWriter<W>,
}

impl<W: Write> Document<W> {
    fn new(writer: BufWriter<W>) -> self {
        Document { doc_type: DocumentType::PS, writer }
    }

    fn add<T: Fabricate>(&mut self, item: &T) -> Result<()> {
        item.fabricate(&mut self.writer)
    }
}

pub fn main() {
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
struct DocumentBuilder<W: Write> {
    doc_type: DocumentType,
    buffer: BufWriter<W>,
}

impl<W: Write> DocumentBuilder<W> {
    fn document_type(&mut self, doc_type: DocumentType) -> self {
        self.doc_type = doc_type;
        self
    }

    fn writer(&mut self, writer: BufWriter<W>) -> self {
        self.writer = writer;
        self
    }

    fn build(self) -> Document {
        Document {
            doc_type: self.doc_type,
            buffer: self.buffer,
        }
    }
}

impl<W: Write> Document<W> {
    fn builder() -> DocumentBuilder {
        let buffer: Vec<u8> = Vec::new();
        let writer = BufWriter::new(&mut buffer);
        DocumentBuilder {
            doc_type: DocumentType::PS,
            buffer: writer,
        }
    }
}

pub fn main() {
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
let path = Path::new("output.ps");
let file = OpenOptions::new()
                .write(true)
                .create(true)
                .open(path)?;
let mut writer = BufWriter::new(&file);
let doc = Document::builder().writer(writer).build();
```
