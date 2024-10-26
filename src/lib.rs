use core::panic;
use std::{
    collections::HashMap,
    io::{BufWriter, Error, Write},
};
mod rect;
pub use rect::Rect;

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
}

impl<W: Write> DocumentBuilder<W> {
    pub fn builder() -> DocumentBuilder<W> {
        DocumentBuilder {
            doc_type: DocumentType::PS,
            buffer: None,
        }
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

    pub fn build(self) -> Document<W> {
        Document {
            doc_type: self.doc_type,
            buffer: Option::expect(
                self.buffer,
                "Write buffer must be set before calling build.",
            ),
        }
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

        registry
    }
}
