use pslib::{Document, Line, Page, Rect, TransformLineOrigin};
use std::{
    fs::{self, OpenOptions},
    io::{BufWriter, Error},
    path::Path,
};

#[test]
fn test1() -> Result<(), Error> {
    let path = Path::new("output/test1.ps");
    if path.exists() {
        let _ = fs::remove_file(path);
    }
    let file = OpenOptions::new().write(true).create(true).open(path)?;
    let writer = BufWriter::new(&file);

    let mut doc = Document::new(writer);
    let mut page = Page::new(400, 400);

    let line = Line::new(100.0, 100.0, 100.0)
        .rotate(45.0)
        .set_orign(TransformLineOrigin::Left)
        .stroke_cmyk(2.0, 1.0, 0.0, 0.0, 0.25);
    let _ = page.add(&line);

    let rect = Rect::new(0.0, 0.0, 100.0, 100.0)
        .fill_rgb(1.0, 0.0, 0.0)
        .stroke_rgb(2.0, 0.0, 0.0, 0.0);
    let _ = page.add(&rect);

    let rect = Rect::new(155.0, 155.0, 100.0, 100.0)
        .fill_rgb(1.0, 0.0, 0.0)
        .rotate(45.0)
        .scale(1.5, 1.0)
        .stroke_rgb(2.0, 0.0, 0.0, 0.0);
    let _ = page.add(&rect);

    let _ = doc.add(&page);
    let _ = doc.close();

    Ok(())
}
