> **Note:** this library is under development. Documentation, functionality, and implementation details are not final.

## Getting Started

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

    // Anything that impls the Fabricate trait can be added to the page
    page.add(line);

    // Add page to document (flushes page buffer to BufWriter)
    doc.add(page);

    // Appends EOF and closes BufWriter
    doc.close();
}
```
