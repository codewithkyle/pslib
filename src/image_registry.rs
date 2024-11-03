use std::{collections::HashMap, path::Path};

pub struct RawImage {
    file_name: String,
    file_path: Box<Path>,
    procedure_name: String,
}
pub struct ImageRegistry {
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

    pub fn add(mut self, path: &Path) -> Self {
        let file_name = path
            .file_name()
            .expect("Unable to determine file name.")
            .to_string_lossy()
            .to_string();
        self.count += 1;
        let proc_name = format!("imager{}", self.count);
        let image = RawImage {
            file_name: file_name.clone(),
            file_path: path.into(),
            procedure_name: proc_name,
        };
        self.images.insert(file_name, image);
        self
    }

    pub fn get_procedure_id(self, file_name: String) -> Option<String> {
        let raw = self.images.get(&file_name);
        if raw.is_none() {
            return None;
        }
        let raw = raw.unwrap();
        Some(raw.procedure_name.clone())
    }
}
