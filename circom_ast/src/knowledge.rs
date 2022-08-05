// Knowledge buckets

#[derive(Copy, Clone, PartialOrd, PartialEq, Ord, Eq)]
pub enum TypeReduction {
    Variable,
    Component,
    Signal,
}

#[derive(Default, Clone)]
pub struct TypeKnowledge {
    reduces_to: Option<TypeReduction>,
}
impl TypeKnowledge {
    pub fn new() -> TypeKnowledge {
        TypeKnowledge::default()
    }
    pub fn set_reduces_to(&mut self, reduces_to: TypeReduction) {
        self.reduces_to = Some(reduces_to);
    }
    pub fn get_reduces_to(&self) -> TypeReduction {
        if let Some(t) = &self.reduces_to {
            *t
        } else {
            panic!("reduces_to knowledge is been look at without being initialized");
        }
    }
    pub fn is_var(&self) -> bool {
        self.get_reduces_to() == TypeReduction::Variable
    }
    pub fn is_component(&self) -> bool {
        self.get_reduces_to() == TypeReduction::Component
    }
    pub fn is_signal(&self) -> bool {
        self.get_reduces_to() == TypeReduction::Signal
    }
}

#[derive(Default, Clone)]
pub struct MemoryKnowledge {
    concrete_dimensions: Option<Vec<usize>>,
    full_length: Option<usize>,
    abstract_memory_address: Option<usize>,
}

impl MemoryKnowledge {
    pub fn new() -> MemoryKnowledge {
        MemoryKnowledge::default()
    }
    pub fn set_concrete_dimensions(&mut self, value: Vec<usize>) {
        self.full_length = Some(value.iter().fold(1, |p, v| p * (*v)));
        self.concrete_dimensions = Some(value);
    }
    pub fn set_abstract_memory_address(&mut self, value: usize) {
        self.abstract_memory_address = Some(value);
    }
    pub fn get_concrete_dimensions(&self) -> &[usize] {
        if let Some(v) = &self.concrete_dimensions {
            v
        } else {
            panic!("concrete dimensions was look at without being initialized");
        }
    }
    pub fn get_full_length(&self) -> usize {
        if let Some(v) = &self.full_length {
            *v
        } else {
            panic!("full dimension was look at without being initialized");
        }
    }
    pub fn get_abstract_memory_address(&self) -> usize {
        if let Some(v) = &self.abstract_memory_address {
            *v
        } else {
            panic!("abstract memory address was look at without being initialized");
        }
    }
}
