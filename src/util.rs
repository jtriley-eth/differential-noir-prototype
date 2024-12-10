use std::collections::BTreeMap;
use std::path::Path;
use std::str::FromStr;

use bn254_blackbox_solver::Bn254BlackBoxSolver;
use fm::FileManager;
use nargo::errors::CompileError;
use nargo::ops::{execute_program, report_errors, DefaultForeignCallExecutor};
use nargo::package::Package;
use nargo::{insert_all_files_for_workspace_into_file_manager, parse_all, prepare_package};
use nargo_toml::{get_package_manifest, resolve_workspace_from_toml, PackageSelection};
use noirc_abi::input_parser::InputValue;
use noirc_driver::{
    check_crate, compile_no_check, CompileOptions, CompiledProgram, CrateName,
    NOIR_ARTIFACT_VERSION_STRING,
};
use noirc_errors::FileDiagnostic;
use noirc_frontend::hir::ParsedFiles;

fn debug_string<E: std::fmt::Debug>(e: E) -> String {
    format!("{e:?}")
}

pub fn noir_fn(
    fn_name: &str,
    input_map: BTreeMap<String, InputValue>,
) -> Result<Option<InputValue>, String> {
    let pkg_name = CrateName::from_str("differential_noir")?;

    let toml_path = get_package_manifest(Path::new(".")).map_err(debug_string)?;

    let workspace = resolve_workspace_from_toml(
        &toml_path,
        PackageSelection::Selected(pkg_name),
        Some(NOIR_ARTIFACT_VERSION_STRING.to_owned()),
    )
    .map_err(debug_string)?;

    let mut workspace_file_manager = workspace.new_file_manager();
    insert_all_files_for_workspace_into_file_manager(&workspace, &mut workspace_file_manager);

    let parsed_files = parse_all(&workspace_file_manager);

    let pkg = workspace
        .into_iter()
        .filter(|pkg| pkg.is_library())
        .next()
        .ok_or("no lib pkg".to_string())?;

    let exported_fns = compile_exported_functions(&workspace_file_manager, &parsed_files, pkg)?;

    let (_, program) = exported_fns
        .into_iter()
        .find(|(name, _)| name == fn_name)
        .ok_or_else(|| "no fn_name".to_string())?;

    let initial_witness = program.abi.encode(&input_map, None).map_err(debug_string)?;

    let solved_witness_stack = execute_program(
        &program.program,
        initial_witness,
        &Bn254BlackBoxSolver,
        &mut DefaultForeignCallExecutor::new(
            true,
            None,
            Some(workspace.root_dir.clone()),
            Some(pkg.name.to_string()),
        ),
    )
    .map_err(debug_string)?;

    let main_witness = &solved_witness_stack
        .peek()
        .ok_or_else(|| "witness peek".to_string())?
        .witness;

    let (_, return_value) = program.abi.decode(main_witness).map_err(debug_string)?;

    Ok(return_value)
}

fn compile_exported_functions(
    file_manager: &FileManager,
    parsed_files: &ParsedFiles,
    pkg: &Package,
) -> Result<Vec<(String, CompiledProgram)>, String> {
    let compile_options = CompileOptions {
        silence_warnings: true,
        ..CompileOptions::default()
    };

    let (mut context, crate_id) = prepare_package(file_manager, parsed_files, pkg);

    report_errors(
        check_crate(&mut context, crate_id, &compile_options),
        &context.file_manager,
        compile_options.deny_warnings,
        compile_options.silence_warnings,
    )
    .map_err(debug_string)?;

    let exported_functions = context.get_all_exported_functions_in_crate(&crate_id);

    exported_functions
        .iter()
        .map(
            |(fn_name, fn_id)| -> Result<(String, CompiledProgram), CompileError> {
                let program =
                    compile_no_check(&mut context, &compile_options, fn_id.clone(), None, false)
                        .map_err(|err| vec![FileDiagnostic::from(err)]);

                let program = report_errors(
                    program.map(|p| (p, Vec::new())),
                    file_manager,
                    compile_options.deny_warnings,
                    compile_options.silence_warnings,
                )?;

                Ok((fn_name.clone(), program))
            },
        )
        .collect::<Result<Vec<(String, CompiledProgram)>, CompileError>>()
        .map_err(debug_string)
}
