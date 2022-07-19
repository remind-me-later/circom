use super::errors::ParseError;
use circom_error::error_definition::Report;
use std::collections::HashSet;
use std::path::PathBuf;

pub struct FileStack {
    current_location: PathBuf,
    black_paths: HashSet<PathBuf>,
    stack: Vec<PathBuf>,
}

impl FileStack {
    pub fn new(src: PathBuf) -> FileStack {
        let mut location = src.clone();
        location.pop();
        FileStack { current_location: location, black_paths: HashSet::new(), stack: vec![src] }
    }

    pub fn add_include(&mut self, path: String) -> Result<(), Report> {
        let mut crr = self.current_location.clone();
        crr.push(&path);

        let path = std::fs::canonicalize(crr)
            .map_err(|_| ParseError::FileOsError { path })
            .map_err(ParseError::produce_report)?;

        if !self.black_paths.contains(&path) {
            self.stack.push(path);
        }

        Ok(())
    }

    pub fn take_next(&mut self) -> Option<PathBuf> {
        loop {
            if let Some(file) = self.stack.pop() {
                if !self.black_paths.contains(&file) {
                    self.current_location = file.clone();
                    self.current_location.pop();
                    self.black_paths.insert(file.clone());
                    return Some(file);
                }
            } else {
                return None;
            }
        }
    }
}
