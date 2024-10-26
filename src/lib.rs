use core::panic;
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

pub trait Fabricate {
    fn fabricate<W: Write>(&self, writer: &mut BufWriter<W>) -> Result<(), Error>;
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

pub struct Document<W: Write> {
    doc_type: DocumentType,
    buffer: BufWriter<W>,
}

impl<W: Write> Document<W> {
    pub fn new(writer: BufWriter<W>) -> Self {
        let mut doc = Document {
            doc_type: DocumentType::PS,
            buffer: writer,
        };
        doc.buffer.write_all(
            format!(
                r#"
                        %!PS-Adobe-3.0
                        %%Creator: pslib {}
                        %%CreationDate: {}
                        %%Pages: (atend)
                        %%EndComments
                    "#,
                env!("CARGO_PKG_VERSION"),
                Utc::now().to_rfc3339()
            )
            .as_bytes(),
        ).unwrap();
        let registry = ProcedureRegistry::with_builtins();
        for procedure in registry.list_procedures() {
            doc.buffer.write_all(procedure.body.as_bytes()).unwrap();
        }
        doc
    }

    pub fn add<T: Fabricate>(&mut self, item: &T) -> Result<(), Error> {
        item.fabricate(&mut self.buffer)
    } 
}

pub struct DocumentBuilder<W: Write> {
    doc_type: DocumentType,
    buffer: Option<BufWriter<W>>,
    width: i32,
    height: i32,
    has_built: bool,
}

impl<W: Write> DocumentBuilder<W> {
    pub fn builder() -> DocumentBuilder<W> {
        DocumentBuilder {
            doc_type: DocumentType::PS,
            buffer: None,
            width: 0,
            height: 0,
            has_built: false,
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
        if !self.has_built {
            panic!("Must call build before calling load_procedures.");
        }
        for procedure in registry.list_procedures() {
            self.buffer
                .as_mut()
                .unwrap_or_else(|| {
                    panic!("Write buffer must be set before calling load_procedures.")
                })
                .write_all(procedure.body.as_bytes())
                .unwrap();
        }
        self
    }

    pub fn build(mut self) -> Document<W> {
        let mut doc = Document {
            doc_type: self.doc_type,
            buffer: Option::expect(
                self.buffer,
                "Write buffer must be set before calling build.",
            ),
        };
        match doc.doc_type {
            DocumentType::PS => {
                doc.buffer.write_all(
                    format!(
                        r#"
                        %!PS-Adobe-3.0
                        %%Creator: pslib {}
                        %%CreationDate: {}
                        %%Pages: (atend)
                        %%EndComments
                    "#,
                        env!("CARGO_PKG_VERSION"),
                        Utc::now().to_rfc3339()
                    )
                    .as_bytes(),
                ).unwrap();
            }
            DocumentType::EPS => {
                doc.buffer.write_all(
                    format!(
                        r#"
                        %!PS-Adobe-3.0 EPSF-3.0
                        %%BoundingBox: 0 0 {} {}
                        %%Creator: pslib {}
                        %%CreationDate: {}
                        %%Pages: 1
                        %%EndComments
                    "#,
                        self.width,
                        self.height,
                        env!("CARGO_PKG_VERSION"),
                        Utc::now().to_rfc3339()
                    )
                    .as_bytes(),
                ).unwrap();
            }
        }
        self.has_built = true;
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
            body: r#"
                /rect {
                    newpath
                    moveto
                    rlineto
                    rlineto
                    rlineto
                    rlineto
                    closepath
                } def
            "#
            .to_string(),
        });

        registry.add_procedure(Procedure {
            name: "line".to_string(),
            body: r#"
                /line {
                    newpath
                    moveto
                    rlineto
                    closepath
                } def
            "#
            .to_string(),
        });

        registry
    }
}
