use super::analyzers::*;
use super::decorators::*;
use circom_error::error_definition::ReportCollection;
use program_structure::program_archive::ProgramArchive;

pub fn check_types(
    program_archive: &mut ProgramArchive,
) -> Result<ReportCollection, ReportCollection> {
    let mut errors = ReportCollection::new();
    let mut warnings = ReportCollection::new();

    // Structural analyses
    program_level_analyses(program_archive, &mut errors).map_err(|_| errors.as_ref())?;
    template_level_analyses(program_archive, &mut errors).map_err(|_| errors.as_ref())?;
    function_level_analyses(program_archive, &mut errors).map_err(|_| errors.as_ref())?;

    // Decorators
    template_level_decorators(program_archive, &mut errors).map_err(|_| errors.as_ref())?;
    function_level_decorators(program_archive, &mut errors).map_err(|_| errors.as_ref())?;

    // Type analysis
    let info = type_check(program_archive, &mut errors).map_err(|_| errors.as_ref())?;

    // Purge unreached definitions
    program_archive.get_mut_functions().retain(|name, _| info.reached.contains(name));
    program_archive.get_mut_templates().retain(|name, _| info.reached.contains(name));

    // Semantic analysis
    semantic_analyses(program_archive, &mut errors, &mut warnings).map_err(|_| errors.as_ref())?;

    Ok(warnings)
}

fn program_level_analyses(
    program_archive: &ProgramArchive,
    reports: &mut ReportCollection,
) -> Result<(), ()> {
    check_naming_correctness(program_archive, reports);

    reports.is_empty().then_some(()).ok_or(())
}

fn template_level_analyses(
    program_archive: &ProgramArchive,
    reports: &mut ReportCollection,
) -> Result<(), ()> {
    for template_data in program_archive.get_templates().values() {
        free_of_returns(template_data, reports);
        check_signal_correctness(template_data, reports);
    }

    reports.is_empty().then_some(()).ok_or(())
}

fn template_level_decorators(
    program_archive: &mut ProgramArchive,
    reports: &mut ReportCollection,
) -> Result<(), ()> {
    component_type_inference::inference(program_archive);

    for template_data in program_archive.get_mut_templates().values_mut() {
        type_reduction::reduce_template(template_data);
    }

    reports.is_empty().then_some(()).ok_or(())
}

fn function_level_analyses(
    program_archive: &ProgramArchive,
    reports: &mut ReportCollection,
) -> Result<(), ()> {
    let functions = program_archive.get_functions();

    for function_data in functions.values() {
        free_of_template_elements(function_data, functions, reports);
        all_paths_with_return_check(function_data, reports);
    }

    reports.is_empty().then_some(()).ok_or(())
}

fn function_level_decorators(
    program_archive: &mut ProgramArchive,
    reports: &mut ReportCollection,
) -> Result<(), ()> {
    for function_data in program_archive.get_mut_functions().values_mut() {
        constants_handler::handle_function_constants(function_data, reports);
        type_reduction::reduce_function(function_data);
    }

    reports.is_empty().then_some(()).ok_or(())
}

fn semantic_analyses(
    program_archive: &ProgramArchive,
    errors: &mut ReportCollection,
    _warnings: &mut ReportCollection,
) -> Result<(), ()> {
    for template_name in program_archive.get_templates().keys() {
        unknown_known_analysis(template_name, program_archive, errors);
        tag_analysis(template_name, program_archive, errors);
    }

    errors.is_empty().then_some(()).ok_or(())
}
