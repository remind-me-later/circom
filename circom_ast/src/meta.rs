use circom_error::file_definition::FileLocation;

use crate::{TypeKnowledge, MemoryKnowledge};

pub trait FillMeta {
    fn fill(&mut self, file_id: usize, elem_id: &mut usize);
}

#[derive(Clone)]
pub struct Meta {
    pub elem_id: usize,
    pub start: usize,
    pub end: usize,
    pub location: FileLocation,
    pub file_id: Option<usize>,
    pub component_inference: Option<String>,
    type_knowledge: TypeKnowledge,
    memory_knowledge: MemoryKnowledge,
}

impl Meta {
    pub fn new(start: usize, end: usize) -> Meta {
        Meta {
            end,
            start,
            elem_id: 0,
            location: start..end,
            file_id: None,
            component_inference: None,
            type_knowledge: TypeKnowledge::default(),
            memory_knowledge: MemoryKnowledge::default(),
        }
    }

    pub fn change_location(&mut self, location: FileLocation, file_id: Option<usize>) {
        self.location = location;
        self.file_id = file_id;
    }

    pub fn get_start(&self) -> usize {
        self.location.start
    }

    pub fn get_end(&self) -> usize {
        self.location.end
    }

    pub fn get_file_id(&self) -> usize {
        if let Some(id) = self.file_id {
            id
        } else {
            panic!("Empty file id accessed")
        }
    }

    pub fn get_memory_knowledge(&self) -> &MemoryKnowledge {
        &self.memory_knowledge
    }

    pub fn get_type_knowledge(&self) -> &TypeKnowledge {
        &self.type_knowledge
    }

    pub fn get_mut_memory_knowledge(&mut self) -> &mut MemoryKnowledge {
        &mut self.memory_knowledge
    }

    pub fn get_mut_type_knowledge(&mut self) -> &mut TypeKnowledge {
        &mut self.type_knowledge
    }

    pub fn file_location(&self) -> FileLocation {
        self.location.clone()
    }

    pub fn set_file_id(&mut self, file_id: usize) {
        self.file_id = Some(file_id);
    }
}
