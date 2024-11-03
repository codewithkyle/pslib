use std::{
    collections::HashMap,
    io::{BufWriter, Error, Write},
};

mod rect;
use chrono::Utc;
pub use rect::Rect;

mod page;
pub use page::Page;

mod line;
pub use line::Line;

mod image_registry;

mod image;

mod inline_image;

pub trait Fabricate {
    fn fabricate<W: Write>(&self, doc_type: &DocumentType, writer: &mut BufWriter<W>) -> Result<(), Error>;
}

pub trait Serialize {
    fn to_postscript_string(&self) -> String;
}

pub enum DocumentType {
    PS,  // PostScript
    EPS, // Encapsulated PostScript
}

pub enum TransformOrigin {
    Center, // Default
    BottomLeft,
    TopLeft,
    TopRight,
    BottomRight,
}

pub enum TransformLineOrigin {
    Left,
    Center, // default
    Right,
}

pub enum ColorMode {
    CMYK,
    RGB,
}

pub enum ImageFit {
    Contain,
    Stretch,
    StretchHorizontal,
    StretchVertical,
    Crop,
}

pub struct Document<W: Write> {
    doc_type: DocumentType,
    buffer: BufWriter<W>,
    page_count: u32,
}

impl<W: Write> Document<W> {
    pub fn new(writer: BufWriter<W>) -> Self {
        let mut doc = Document {
            doc_type: DocumentType::PS,
            buffer: writer,
            page_count: 0,
        };
        doc.buffer
            .write_all(
                format!(
                    r#"%!PS-Adobe-3.0
%%Creator: pslib {}
%%CreationDate: {}
%%Pages: (atend)
%%EndComments
"#,
                    env!("CARGO_PKG_VERSION"),
                    Utc::now().to_rfc3339()
                )
                .as_bytes(),
            )
            .unwrap();
        let registry = ProcedureRegistry::with_builtins();
        for procedure in registry.list_procedures() {
            doc.buffer.write_all(procedure.body.as_bytes()).unwrap();
            doc.buffer.write_all("\n".as_bytes()).unwrap();
        }
        doc
    }

    pub fn add<T: Fabricate>(&mut self, item: &T) -> Result<(), Error> {
        match self.doc_type {
            DocumentType::PS => {
                self.page_count += 1;
                self.buffer.write_all(
                    format!("%%Page: {} {}\n", self.page_count, self.page_count).as_bytes(),
                )?;
            }
            _ => {}
        }
        item.fabricate(&self.doc_type, &mut self.buffer)
    }

    pub fn close(mut self) -> Result<(), Error> {
        self.buffer.write_all("%%EOF".as_bytes())?;
        self.buffer.flush()?;
        Ok(())
    }
}

pub struct DocumentBuilder<W: Write> {
    doc_type: DocumentType,
    buffer: Option<BufWriter<W>>,
    width: i32,
    height: i32,
    registry: ProcedureRegistry,
}

impl<W: Write> DocumentBuilder<W> {
    pub fn builder() -> DocumentBuilder<W> {
        DocumentBuilder {
            doc_type: DocumentType::PS,
            buffer: None,
            width: 0,
            height: 0,
            registry: ProcedureRegistry::new(),
        }
    }

    pub fn bounding_box(mut self, width: i32, height: i32) -> Self {
        self.width = width.max(1);
        self.height = height.max(1);
        self
    }

    pub fn document_type(mut self, doc_type: DocumentType) -> Self {
        self.doc_type = doc_type;
        self
    }

    pub fn writer(mut self, writer: BufWriter<W>) -> Self {
        self.buffer = Some(writer);
        self
    }

    pub fn load_procedures(mut self, registry: ProcedureRegistry) -> Self {
        self.registry = registry;
        self
    }

    pub fn build(self) -> Document<W> {
        let mut doc = Document {
            doc_type: self.doc_type,
            buffer: Option::expect(
                self.buffer,
                "Write buffer must be set before calling build.",
            ),
            page_count: 0,
        };
        match doc.doc_type {
            DocumentType::PS => {
                doc.buffer
                    .write_all(
                        format!(
                            r#"%!PS-Adobe-3.0
%%Creator: pslib {}
%%CreationDate: {}
%%Pages: (atend)
%%EndComments
"#,
                            env!("CARGO_PKG_VERSION"),
                            Utc::now().to_rfc3339()
                        )
                        .as_bytes(),
                    )
                    .unwrap();
            }
            DocumentType::EPS => {
                doc.buffer
                    .write_all(
                        format!(
                            r#"%!PS-Adobe-3.0 EPSF-3.0
%%BoundingBox: 0 0 {} {}
%%Creator: pslib {}
%%CreationDate: {}
%%EndComments
"#,
                            self.width,
                            self.height,
                            env!("CARGO_PKG_VERSION"),
                            Utc::now().to_rfc3339()
                        )
                        .as_bytes(),
                    )
                    .unwrap();
            }
        }
        for procedure in self.registry.list_procedures() {
            doc.buffer.write_all(procedure.body.as_bytes()).unwrap();
            doc.buffer.write_all("\n".as_bytes()).unwrap();
        }
        doc
    }
}

pub struct Procedure {
    pub name: String,
    pub body: String,
}

pub struct ProcedureRegistry {
    procedures: HashMap<String, Procedure>,
}

impl ProcedureRegistry {
    pub fn new() -> Self {
        ProcedureRegistry {
            procedures: HashMap::new(),
        }
    }

    pub fn add_procedure(&mut self, procedure: Procedure) {
        self.procedures.insert(procedure.name.clone(), procedure);
    }

    pub fn get_procedure(&self, name: &str) -> Option<&Procedure> {
        self.procedures.get(name)
    }

    pub fn list_procedures(&self) -> Vec<&Procedure> {
        self.procedures.values().collect()
    }

    pub fn with_builtins() -> Self {
        let mut registry = Self::new();

        registry.add_procedure(Procedure {
            name: "rect".to_string(),
            body: r#"/rect { newpath moveto rlineto rlineto rlineto rlineto closepath } def"#
            .to_string(),
        });

        registry.add_procedure(Procedure {
            name: "line".to_string(),
            body: r#"/line { newpath moveto rlineto closepath } def"#
            .to_string(),
        });

        registry.add_procedure(Procedure {
            name: "fill_rgb".to_string(),
            body: r#"/fillrgb { gsave setrgbcolor fill grestore } def"#
            .to_string(),
        });

        registry.add_procedure(Procedure {
            name: "fill_cmyk".to_string(),
            body: r#"/fillcmyk { gsave setcmykcolor fill grestore } def"#
            .to_string(),
        });

        registry.add_procedure(Procedure {
            name: "stroke_rgb".to_string(),
            body: r#"/strokergb { gsave setlinewidth setrgbcolor stroke grestore } def"#
            .to_string(),
        });

        registry.add_procedure(Procedure {
            name: "stroke_cmyk".to_string(),
            body: r#"/strokecmyk { gsave setlinewidth setcmykcolor stroke grestore } def"#
            .to_string(),
        });

        registry
    }
}
