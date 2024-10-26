use std::io::{BufWriter, Error, Write};

use crate::{Fabricate, Serialize};

pub struct Page {
    width: i32,
    height: i32,
    buffer: Vec<u8>,
}

impl Page {
    pub fn new(width: i32, height: i32) -> Self {
        Page {
            width: width.max(1),
            height: height.max(1),
            buffer: Vec::new(),
        }
    }

    pub fn add<T: Serialize>(&mut self, item: &T) -> Result<(), Error> {
        self.buffer
            .write_all(item.to_postscript_string().as_bytes())?;
        Ok(())
    }
}

impl Fabricate for Page {
    fn fabricate<W: Write>(&self, writer: &mut BufWriter<W>) -> Result<(), Error> {
        write!(
            writer,
            r#"%%PageBoundingBox: 0 0 {} {}
<< /PageSize [{} {}] >> setpagedevice
"#,
            self.width, self.height, self.width, self.height
        )?;
        writer.write_all(&self.buffer)?;
        writer.write_all("showpage\n".as_bytes())?;
        Ok(())
    }
}
