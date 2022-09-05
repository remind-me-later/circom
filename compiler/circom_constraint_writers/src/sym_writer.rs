use std::fs::File;
use std::io::{self, BufWriter, Write};

pub struct SymElem {
    pub original: i64,
    pub witness: i64,
    pub node_id: i64,
    pub symbol: String,
}
impl ToString for SymElem {
    fn to_string(&self) -> String {
        format!(
            "{},{},{},{}",
            self.original, self.witness, self.node_id, self.symbol
        )
    }
}

pub struct SymFile {
    writer: BufWriter<File>,
}

impl SymFile {
    pub fn new(file: &str) -> io::Result<SymFile> {
        let file = File::create(file)?;
        let writer = BufWriter::new(file);
        Ok(SymFile { writer })
    }

    pub fn write_sym_elem(sym: &mut SymFile, elem: SymElem) -> io::Result<()> {
        sym.writer.write_all(elem.to_string().as_bytes())?;
        sym.writer.write_all(b"\n")?;
        sym.writer.flush()
    }

    pub fn close(_sym: SymFile) {}
}
