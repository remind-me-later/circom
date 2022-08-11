use std::fs::File;
use std::io::{self, BufWriter, Write};

pub struct ConstraintJSON {
    writer_constraints: BufWriter<File>,
    constraints_flag: bool,
}

impl ConstraintJSON {
    pub fn new(file: &str) -> io::Result<ConstraintJSON> {
        let file_constraints = File::create(file)?;
        let mut writer_constraints = BufWriter::new(file_constraints);

        writer_constraints.write_all(b"{")?;
        writer_constraints.flush()?;
        writer_constraints.write_all(b"\n\"constraints\": [")?;
        writer_constraints.flush()?;

        Ok(ConstraintJSON {
            writer_constraints,
            constraints_flag: false,
        })
    }
    pub fn write_constraint(&mut self, constraint: &str) -> io::Result<()> {
        if !self.constraints_flag {
            self.constraints_flag = true;
            self.writer_constraints.write_all(b"\n")?;
            self.writer_constraints.flush()?;
        } else {
            self.writer_constraints.write_all(b",\n")?;
            self.writer_constraints.flush()?;
        }
        self.writer_constraints.write_all(constraint.as_bytes())?;
        self.writer_constraints.flush()?;
        Ok(())
    }
    pub fn end(mut self) -> io::Result<()> {
        self.writer_constraints.write_all(b"\n]\n}")?;
        self.writer_constraints.flush()?;
        Ok(())
    }
}

pub struct SignalsJSON {
    writer_signals: BufWriter<File>,
}
impl SignalsJSON {
    pub fn new(file: &str) -> io::Result<SignalsJSON> {
        let file_signals = File::create(file)?;
        let mut writer_signals = BufWriter::new(file_signals);
        writer_signals.write_all(b"{")?;
        writer_signals.flush()?;
        writer_signals.write_all(b"\n\"signalName2Idx\": {")?;
        writer_signals.flush()?;
        writer_signals.write_all(b"\n\"one\" : \"0\"")?;
        writer_signals.flush()?;
        Ok(SignalsJSON { writer_signals })
    }
    pub fn write_correspondence(&mut self, signal: String, data: String) -> io::Result<()> {
        self.writer_signals
            .write_all(format!(",\n\"{}\" : {}", signal, data).as_bytes())?;
        self.writer_signals.flush()
    }
    pub fn end(mut self) -> io::Result<()> {
        self.writer_signals.write_all(b"\n}\n}")?;
        self.writer_signals.flush()
    }
}

pub struct SubstitutionJSON {
    writer_substitutions: BufWriter<File>,
    first: bool,
}
impl SubstitutionJSON {
    pub fn new(file: &str) -> io::Result<SubstitutionJSON> {
        let first = true;
        let file_substitutions = File::create(file)?;
        let mut writer_substitutions = BufWriter::new(file_substitutions);
        writer_substitutions.write_all(b"{")?;
        writer_substitutions.flush()?;
        writer_substitutions.write_all(b"\n\"substitution\": {")?;
        writer_substitutions.flush()?;
        Ok(SubstitutionJSON {
            writer_substitutions,
            first,
        })
    }
    pub fn write_substitution(&mut self, signal: &str, substitution: &str) -> io::Result<()> {
        if self.first {
            self.first = false;
            self.writer_substitutions.write_all(b"\n")?;
        } else {
            self.writer_substitutions.write_all(b",\n")?;
        }
        let substitution = format!("\"{}\" : {}", signal, substitution);
        self.writer_substitutions.flush()?;
        self.writer_substitutions
            .write_all(substitution.as_bytes())?;
        self.writer_substitutions.flush()?;
        Ok(())
    }
    pub fn end(mut self) -> io::Result<()> {
        self.writer_substitutions.write_all(b"\n}\n}")?;
        self.writer_substitutions.flush()
    }
}
