mod errors;
mod include_logic;
mod parser_logic;
mod test;

use errors::ParseError;
use include_logic::FileStack;
use circom_error::error_definition::{Report, ReportCollection};
use circom_error::file_definition::{FileLibrary};
use circom_program_structure::program_archive::ProgramArchive;
use circom_program_structure::ast::Version;
use std::path::PathBuf;
use std::str::FromStr;
use lalrpop_util::lalrpop_mod;

lalrpop_mod!(pub lang);

pub fn run_parser(
    file: String,
    version: &str,
) -> Result<(ProgramArchive, ReportCollection), (FileLibrary, ReportCollection)> {
    let mut file_library = FileLibrary::new();
    let mut definitions = Vec::new();
    let mut main_components = Vec::new();
    let mut file_stack = FileStack::new(PathBuf::from(file));
    let mut warnings = Vec::new();

    while let Some(crr_file) = file_stack.take_next() {
        let (path, src) = open_file(crr_file).map_err(|e| (file_library.clone(), vec![e]))?;
        let file_id = file_library.add_file(path.clone(), src.clone());
        let program =
            parser_logic::parse_file(&src, file_id).map_err(|e| (file_library.clone(), vec![e]))?;

        if let Some(main) = program.main_component {
            main_components.push((file_id, main));
        }
        let includes = program.includes;
        definitions.push((file_id, program.definitions));
        for include in includes {
            file_stack.add_include(include).map_err(|e| (file_library.clone(), vec![e]))?;
        }
        warnings.append(
            &mut check_number_version(
                path,
                program.compiler_version,
                parse_number_version(version),
            )
            .map_err(|e| (file_library.clone(), vec![e]))?,
        );
    }

    if main_components.is_empty() {
        let report = ParseError::NoMainError.produce_report();
        Err((file_library, vec![report]))
    } else if main_components.len() > 1 {
        let report = ParseError::MultipleMainError.produce_report();
        Err((file_library, vec![report]))
    } else {
        let (main_id, main_component) = main_components.pop().unwrap();
        let result_program_archive =
            ProgramArchive::new(file_library, main_id, main_component, definitions);
        match result_program_archive {
            Err((lib, rep)) => Err((lib, rep)),
            Ok(program_archive) => Ok((program_archive, warnings)),
        }
    }
}

fn open_file(path: PathBuf) -> Result<(String, String), Report> /* path, src*/ {
    use ParseError::FileOsError;
    use std::fs::read_to_string;
    let path_str = format!("{:?}", path);
    read_to_string(path)
        .map(|contents| (path_str.clone(), contents))
        .map_err(|_| FileOsError { path: path_str.clone() })
        .map_err(ParseError::produce_report)
}

fn parse_number_version(version: &str) -> Version {
    let version_splitted: Vec<&str> = version.split('.').collect();

    Version {
        version: usize::from_str(version_splitted[0]).unwrap(),
        subversion: usize::from_str(version_splitted[1]).unwrap(),
        subsubversion: usize::from_str(version_splitted[2]).unwrap(),
    }
}

fn check_number_version(
    file_path: String,
    version_file: Option<Version>,
    version_compiler: Version,
) -> Result<ReportCollection, Report> {
    use errors::{ParseError::CompilerVersionError, ParseError::NoCompilerVersionWarning};
    if let Some(required_version) = version_file {
        if required_version.version == version_compiler.version
            && required_version.subversion == version_compiler.subversion
            && required_version.subsubversion <= version_compiler.subsubversion
        {
            Ok(vec![])
        } else {
            Err(ParseError::produce_report(CompilerVersionError {
                path: file_path,
                required_version,
                version: version_compiler,
            }))
        }
    } else {
        let report = ParseError::produce_report(NoCompilerVersionWarning {
            path: file_path,
            version: version_compiler,
        });
        Ok(vec![report])
    }
}
