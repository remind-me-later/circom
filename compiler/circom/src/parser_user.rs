use super::input_user::Input;
use crate::VERSION;
use circom_error::error_definition::Report;
use program_structure::program_archive::ProgramArchive;

pub fn parse_project(input_info: &Input) -> Result<ProgramArchive, ()> {
    let initial_file = input_info.input_file();
    let result_program_archive = parser::run_parser(initial_file, VERSION);
    match result_program_archive {
        Err((file_library, report_collection)) => {
            Report::print_reports(&report_collection, &file_library);
            Err(())
        }
        Ok((program_archive, warnings)) => {
            Report::print_reports(&warnings, &program_archive.file_library);
            Ok(program_archive)
        }
    }
}
