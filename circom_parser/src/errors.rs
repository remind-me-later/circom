use circom_error::error_code::ReportCode;
use circom_error::error_definition::Report;
use circom_error::file_definition::{FileID, FileLocation};
use circom_program_structure::abstract_syntax_tree::ast::Version;

pub enum ParseError {
    UnclosedCommentError { location: FileLocation, file_id: FileID },
    ParsingError { location: FileLocation, file_id: FileID, msg: String },
    FileOsError { path: String },
    NoMainError,
    MultipleMainError,
    CompilerVersionError { path: String, required_version: Version, version: Version },
    NoCompilerVersionWarning { path: String, version: Version },
}

impl ParseError {
    pub fn produce_report(self) -> Report {
        match self {
            ParseError::UnclosedCommentError { location, file_id } => {
                let mut report =
                    Report::error("unterminated /* */".to_string(), ReportCode::UnclosedComment);
                report.add_primary(location, file_id, "Comment starts here".to_string());
                report
            }
            ParseError::ParsingError { location, file_id, msg } => {
                let mut report = Report::error(msg, ReportCode::ParseFail);
                report.add_primary(location, file_id, "Invalid syntax".to_string());
                report
            }
            ParseError::FileOsError { path } => Report::error(
                format!("Could not open file {}", path),
                ReportCode::FileOsError,
            ),
            ParseError::NoMainError => Report::error(
                "No main specified in the project structure".to_string(),
                ReportCode::NoMainFoundInProject,
            ),
            ParseError::MultipleMainError => Report::error(
                "Multiple main components in the project structure".to_string(),
                ReportCode::MultipleMainInComponent,
            ),
            ParseError::CompilerVersionError { path, required_version, version } => 
            Report::error(
                             format!("File {} requires pragma version {:?} that is not supported by the compiler (version {:?})", path, required_version, version ),
                             ReportCode::CompilerVersionError,
                        ),
            ParseError::NoCompilerVersionWarning { path, version } => {
                Report::warning(
                                 format!(
                                     "File {} does not include pragma version. Assuming pragma version {:?}",
                                     path, version
                                 ),
                                 ReportCode::NoCompilerVersionWarning,
                             )
            }
        }
    }
}
