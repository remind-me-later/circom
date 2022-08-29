mod errors;
mod include_logic;
mod parser_logic;

lalrpop_mod!(pub lang);

use circom_ast::Version;
use circom_error::error_code::ReportCode;
use circom_error::error_definition::{Report, ReportCollection};
use circom_error::file_definition::FileLibrary;
use errors::Error;
use include_logic::{FileStack, IncludesGraph};
use lalrpop_util::lalrpop_mod;
use program_structure::program_archive::ProgramArchive;
use std::path::Path;
use std::str::FromStr;

pub fn run_parser(
    file: &str,
    version: &str,
) -> Result<(ProgramArchive, ReportCollection), (FileLibrary, ReportCollection)> {
    let mut file_library = FileLibrary::new();
    let mut definitions = Vec::new();
    let mut main_components = Vec::new();
    let mut file_stack = FileStack::new(file);
    let mut includes_graph = IncludesGraph::new();
    let mut warnings = Vec::new();

    while let Some(crr_file) = FileStack::take_next(&mut file_stack) {
        let (path, src) = open_file(&crr_file).map_err(|e| (file_library.clone(), vec![e]))?;
        let file_id = file_library.add_file(path.clone(), src.clone());
        let program =
            parser_logic::parse_file(&src, file_id).map_err(|e| (file_library.clone(), e))?;

        if let Some(main) = program.main_component {
            main_components.push((file_id, main));
        }
        includes_graph.add_node(
            crr_file,
            program.custom_gates,
            program.custom_gates_declared,
        );
        let includes = program.includes;
        definitions.push((file_id, program.definitions));
        for include in includes {
            FileStack::add_include(&mut file_stack, include.clone())
                .map_err(|e| (file_library.clone(), vec![e]))?;
            includes_graph
                .add_edge(include)
                .map_err(|e| (file_library.clone(), vec![e]))?;
        }
        warnings.append(
            &mut check_number_version(
                &path,
                program.compiler_version,
                parse_number_version(version),
            )
            .map_err(|e| (file_library.clone(), vec![e]))?,
        );
    }

    if main_components.is_empty() {
        let report = Error::NoMain.produce_report();
        Err((file_library, vec![report]))
    } else if main_components.len() > 1 {
        let report = Error::MultipleMain.produce_report();
        Err((file_library, vec![report]))
    } else {
        let errors: ReportCollection = includes_graph
            .get_problematic_paths()
            .iter()
            .map(|path| {
                Report::error(
                    format!(
                        "Missing custom gates' pragma in the following chain of includes {}",
                        IncludesGraph::display_path(path)
                    ),
                    ReportCode::CustomGatesPragmaError,
                )
            })
            .collect();
        if !errors.is_empty() {
            Err((file_library, errors))
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
}

fn open_file(path: &Path) -> Result<(String, String), Report> /* path, src */ {
    use std::fs::read_to_string;

    read_to_string(path)
        .map(|contents| (path.as_os_str().to_str().unwrap().to_string(), contents))
        .map_err(|_| Error::FileOs {
            path: path.as_os_str().to_str().unwrap().to_string(),
        })
        .map_err(Error::produce_report)
}

fn parse_number_version(version: &str) -> Version {
    let version_splitted: Vec<&str> = version.split('.').collect();
    Version::new(
        usize::from_str(version_splitted[0]).unwrap(),
        usize::from_str(version_splitted[1]).unwrap(),
        usize::from_str(version_splitted[2]).unwrap(),
    )
}

fn check_number_version(
    file_path: &str,
    version_file: Option<Version>,
    version_compiler: Version,
) -> Result<ReportCollection, Report> {
    use errors::NoCompilerVersionWarning;
    if let Some(required_version) = version_file {
        if required_version.major() == version_compiler.major()
            && (required_version.minor() < version_compiler.minor()
                || (required_version.minor() == version_compiler.minor()
                    && required_version.patch() <= version_compiler.patch()))
        {
            Ok(vec![])
        } else {
            Err(Error::CompilerVersion {
                path: file_path.to_string(),
                required_version,
                version: version_compiler,
            }
            .produce_report())
        }
    } else {
        let report = NoCompilerVersionWarning::produce_report(NoCompilerVersionWarning {
            path: file_path.to_string(),
            version: version_compiler,
        });
        Ok(vec![report])
    }
}
