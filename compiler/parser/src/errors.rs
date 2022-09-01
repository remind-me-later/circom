use circom_ast::Version;
use circom_error::error_code::ReportCode;
use circom_error::error_definition::Report;
use circom_error::file_definition::{FileID, LocationInFile};

pub enum Error {
    UnclosedComment {
        location: LocationInFile,
        file_id: FileID,
    },
    GenericParsing {
        location: LocationInFile,
        file_id: FileID,
        msg: String,
    },
    FileOs {
        path: String,
    },
    NoMain,
    MultipleMain,
    CompilerVersion {
        path: String,
        required_version: Version,
        version: Version,
    },
    MissingSemicolon,
    EmptyInclude,
    MissingVersion,
    EmptyPragma,
}

impl Error {
    pub fn produce_report(self) -> Report {
        match self {
            Error::UnclosedComment { location, file_id } => {
                let mut report =
                    Report::error("unterminated /* */".to_string(), ReportCode::ParseFail);
                report.add_primary(location, file_id, "Comment starts here".to_string());
                report
            }
            Error::GenericParsing {
                location,
                file_id,
                msg,
            } => {
                let mut report = Report::error(msg, ReportCode::ParseFail);
                report.add_primary(location, file_id, "Invalid syntax".to_string());
                report
            }
            Error::FileOs { path } => Report::error(
                format!("Could not open file {}", path),
                ReportCode::ParseFail,
            ),
            Error::NoMain => Report::error(
                "No main specified in the project structure".to_string(),
                ReportCode::NoMainFoundInProject,
            ),
            Error::MultipleMain =>{
                Report::error(
                    "Multiple main components in the project structure".to_string(),
                    ReportCode::MultipleMainInComponent,
                )
            }
            Error::CompilerVersion {
                path,
                required_version,
                version,
            } => {
                Report::error(
                    format!("File {} requires pragma version {:?} that is not supported by the compiler (version {:?})", path, required_version, version ),
                    ReportCode::CompilerVersionError,
                )
            }
            Error::MissingSemicolon => {
                Report::error("missing semicolon".to_owned(), ReportCode::ParseFail)
            }
            Error::EmptyInclude => {
                Report::error("empty include".to_owned(), ReportCode::ParseFail)
            }
            Error::MissingVersion => Report::error("missing version in pragma circom".to_owned(), ReportCode::ParseFail),
            Error::EmptyPragma => Report::error("empty pragma".to_owned(), ReportCode::ParseFail),
        }
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
