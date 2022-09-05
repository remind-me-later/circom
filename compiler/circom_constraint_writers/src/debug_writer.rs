use std::io;

use super::json_writer::ConstraintJSON;

#[derive(Clone)]
pub struct DebugWriter {
    pub json_constraints: String,
}
impl DebugWriter {
    pub fn new(c: String) -> io::Result<DebugWriter> {
        Ok(DebugWriter {
            json_constraints: c,
        })
    }

    pub fn build_constraints_file(&self) -> io::Result<ConstraintJSON> {
        ConstraintJSON::new(&self.json_constraints)
    }
}
