use program_structure::error_code::ReportCode;
use program_structure::error_definition::Report;
use program_structure::file_definition::{FileID, FileLocation};
use program_structure::abstract_syntax_tree::ast::Version;

pub struct UnclosedCommentError {
    pub location: FileLocation,
    pub file_id: FileID,
}

impl UnclosedCommentError {
    pub fn produce_report(error: Self) -> Report {
        let mut report = Report::error("unterminated /* */".to_string(), ReportCode::ParseFail);
        report.add_primary(error.location, error.file_id, "Comment starts here".to_string());
        report
    }
}

pub struct ParsingError {
    pub location: FileLocation,
    pub file_id: FileID,
    pub msg: String,
}

impl ParsingError {
    pub fn produce_report(error: Self) -> Report {
        let mut report = Report::error(error.msg, ReportCode::ParseFail);
        report.add_primary(error.location, error.file_id, "Invalid syntax".to_string());
        report
    }
}

pub struct FileOsError {
    pub path: String,
}

impl FileOsError {
    pub fn produce_report(error: Self) -> Report {
        Report::error(format!("Could not open file {}", error.path), ReportCode::ParseFail)
    }
}

pub struct NoMainError;

impl NoMainError {
    pub fn produce_report() -> Report {
        Report::error(
            "No main specified in the project structure".to_string(),
            ReportCode::NoMainFoundInProject,
        )
    }
}

pub struct MultipleMainError;

impl MultipleMainError {
    pub fn produce_report() -> Report {
        Report::error(
            "Multiple main components in the project structure".to_string(),
            ReportCode::MultipleMainInComponent,
        )
    }
}

pub struct CompilerVersionError {
    pub path: String,
    pub required_version: Version,
    pub version: Version,
}

impl CompilerVersionError {
    pub fn produce_report(error: Self) -> Report {
        Report::error(
            format!("File {} requires pragma version {:?} that is not supported by the compiler (version {:?})", error.path, error.required_version, error.version ),
            ReportCode::CompilerVersionError,
        )
    }
}

pub struct NoCompilerVersionWarning {
    pub path: String,
    pub version: Version,
}

impl NoCompilerVersionWarning {
    pub fn produce_report(error: Self) -> Report {
        Report::warning(
            format!(
                "File {} does not include pragma version. Assuming pragma version {:?}",
                error.path, error.version
            ),
            ReportCode::NoCompilerVersionWarning,
        )
    }
}
