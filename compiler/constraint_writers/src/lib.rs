use std::io;

pub mod debug_writer;
pub mod json_writer;
pub mod log_writer;
pub mod r1cs_writer;
pub mod sym_writer;

pub trait ConstraintExporter {
    fn r1cs(&self, out: &str) -> io::Result<()>;
    fn json_constraints(&self, writer: &debug_writer::DebugWriter) -> io::Result<()>;
    fn sym(&self, out: &str) -> io::Result<()>;
}
