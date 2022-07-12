//! Functions used to manage the build workspace and build the runner with the provided probe-rs code
use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::{bail, Result};
use cargo_toml::{Dependency, DependencyDetail, Manifest};
use fs_extra::dir::CopyOptions;
use ignore::WalkBuilder;
use thiserror::Error;

use super::config;

/// Link to the runner source repository
const RUNNER_SOURCE_REPO: &str = "https://github.com/probe-rs/hive-software.git";

/// Errors which happen if the provided probe-rs project is not correct
#[derive(Debug, Error)]
pub(super) enum WorkspaceError {
    #[error("No cargofile found in probe-rs folder")]
    NoProbeRsCargoFile,
    #[error("No hive directory with testfunctions found in provided probe-rs tests folder")]
    NoHiveDir,
    #[error("Failed to clone the runner source code into the workspace.\n\nGit output:\n{0}")]
    CloneError(String),
    #[error("Failed to build the test runner.\n\nCargo build output:\n{0}")]
    BuildError(String),
}

/// Unpack the provided probe-rs tarball into the workspace and check if it is a valid probe-rs project.
/// If the checks succeed, the hive.rs file in the tests folder of the probe-rs project is copied into the runner for compilation.
///
/// # Panics
/// If the [`WORKSPACE_PATH`] does not exist. This means that the environment in which the monitor runs in has not been configured properly or if removing the cargofile of the provided tarball fails which is likely a permission issue.
pub(super) fn prepare_workspace(probe_rs_project: &Path) -> Result<()> {
    let project_dirs = config::get_project_dirs();
    let workspace_path = project_dirs.cache_dir();

    let project_path = workspace_path.join("probe-rs-testcandidate");

    if !workspace_path.exists() {
        fs::create_dir_all(workspace_path).expect("Failed to create cache directory for Hive CLI");
    }

    clone_runner_source(workspace_path)?;

    copy_testcandidate_source_to_workspace(probe_rs_project, &project_path)?;

    remove_testcandidate_workspace_cargofile(&project_path)?;

    modify_testcandidate_probe_rs_cargofile(&project_path)?;

    copy_hive_test_dir(&project_path, &workspace_path)?;

    Ok(())
}

/// Clone the source code of the runner into the workspace if the workspace does not exist yet
///
/// # Panics
/// If invoking the git command fails which means that git is not accessible to the application (possibly not installed)
fn clone_runner_source(workspace_path: &Path) -> Result<()> {
    if workspace_path.exists() {
        //TODO: This is a very stupid workaround and does not fix the problem of detecting an existing git repo
        return Ok(());
    }

    let clone_output = Command::new("git")
        .args([
            "clone",
            RUNNER_SOURCE_REPO,
            workspace_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to run git cone. Is git installed and accessible to the application?");

    if !clone_output.status.success() {
        return Err(WorkspaceError::CloneError(
            String::from_utf8(clone_output.stderr)
                .unwrap_or_else(|_| "Could not parse git clone output to utf8".to_owned()),
        )
        .into());
    }

    Ok(())
}

/// Copies the probe-rs testcandidate source code into the workspace folder. It respects any git rules like .gitignore during the copy process.
///
/// # Panics
/// In case any file system operations fail which indicate insufficient permissions
fn copy_testcandidate_source_to_workspace(
    probe_rs_project: &Path,
    project_path: &Path,
) -> Result<()> {
    let mut copy_paths = vec![];

    for result in WalkBuilder::new(&probe_rs_project)
        .require_git(false)
        .build()
    {
        match result {
            Ok(entry) => {
                let relative_path = pathdiff::diff_paths(entry.path(), &probe_rs_project).unwrap();

                if relative_path == Path::new("") {
                    // Ignore basepath as it does not add anything to the archive
                    continue;
                }

                copy_paths.push((entry.path().to_owned(), relative_path));
            }
            Err(err) => bail!("Failed to read project files: {}", err),
        }
    }

    for (from, to_relative) in copy_paths {
        if from.is_dir() {
            fs::create_dir_all(project_path.join(to_relative)).expect("Failed to copy files from probe-rs testcandidate source to workspace. This is likely due to missing permissions.");
            continue;
        }

        fs::copy(from, project_path.join(to_relative)).expect("Failed to copy files from probe-rs testcandidate source to workspace. This is likely due to missing permissions.");
    }

    Ok(())
}

/// Deletes the workspace Cargo.toml of the provided probe-rs project to avoid nested workspaces which would fail the build process
///
/// # Panics
/// In case any file system operations fail which indicate insufficient permissions or a corrupted install
fn remove_testcandidate_workspace_cargofile(project_path: &Path) -> Result<()> {
    let cargofile_path = project_path.join("Cargo.toml");

    fs::remove_file(cargofile_path).expect("Failed to delete workspace cargofile of probe-rs testcandidate. This is likely caused by insufficient permissions");

    Ok(())
}

/// Modifies the probe_rs testcandidate crates cargofile hive-test dependency to depend on the local hive-test source on the testserver instead of any other source defined by the user.
///
/// This is required to avoid any circular dependencies or using outdated/incompatible versions on the testserver as well as allowing the build process to pass in case of custom path dependencies.
///
/// # Panics
/// In case any file system operations fail which indicate insufficient permissions or a corrupted install
fn modify_testcandidate_probe_rs_cargofile(project_path: &Path) -> Result<()> {
    let cargofile_path = project_path.join("probe-rs/Cargo.toml");

    if !cargofile_path.exists() {
        return Err(WorkspaceError::NoProbeRsCargoFile.into());
    }

    let mut manifest = Manifest::from_path(&cargofile_path)?;

    if let Some(hive_test) = manifest.dev_dependencies.get_mut("hive-test") {
        let mut hive_test_dependency = DependencyDetail::default();

        hive_test_dependency.package = Some("hive-test".to_owned());
        hive_test_dependency.path = Some("../../hive-test/".to_owned());

        *hive_test = Dependency::Detailed(hive_test_dependency);

        fs::write(cargofile_path, toml::to_string_pretty(&manifest).unwrap()).expect("Failed to write modified probe-rs Cargo.toml file. This is likely due to a corrupted installation or missing permissions.");
    }

    Ok(())
}

/// Copy the hive test directory of probe-rs tests into the runner source
///
/// # Panics
/// In case any file system operations fail which indicate insufficient permissions or a corrupted install
fn copy_hive_test_dir(project_path: &Path, workspace_path: &Path) -> Result<()> {
    let hive_path = project_path.join("probe-rs/tests/hive/");

    if !hive_path.exists() {
        return Err(WorkspaceError::NoHiveDir.into());
    }

    let mut copy_options = CopyOptions::new();
    copy_options.overwrite = true;
    copy_options.copy_inside = true;

    let runner_hive_path = workspace_path.join("runner/src/");
    fs_extra::copy_items(&[hive_path], runner_hive_path, &copy_options).expect("Failed to copy hive directory from probe-rs testcandidate to runner source files. This is likely due to missing permissions.");

    Ok(())
}

/// Cleans the workspace by deleting the testcandidate code but leaving the targets build directory in place
///
/// # Panics
/// If deleting the files fails which is usually caused by a corrupted installation or missing permissions
pub fn clean_workspace() {
    let project_dirs = config::get_project_dirs();
    let workspace_path = project_dirs.cache_dir();

    let project_path = workspace_path.join("probe-rs-testcandidate");

    let testcandidate_contents = fs::read_dir(project_path).expect(
        "Failed to read probe-rs-testcandidate directory. This might be caused by missing permissions.",
    );

    for entry in testcandidate_contents.flatten() {
        let path = entry.path();
        if path.is_dir() {
            fs::remove_dir_all(path).expect("Failed to delete directory on workspace cleanup. This might be caused by missing permissions.");
        } else {
            fs::remove_file(path).expect("Failed to delete file on workspace cleanup. This might be caused by missing permissions.")
        }
    }
}

/// Builds the runner binary with the provided probe-rs test dependency using Cross.
///
/// Cross is used as the hive cli can be used on various different platforms, therefore cross-compilation is necessary to aarch64
///
/// # Panics
/// If invoking the cross command fails which means that cross is not accessible to the application (possibly not installed)
pub fn build_runner() -> Result<Vec<u8>> {
    let project_dirs = config::get_project_dirs();
    let workspace_path = project_dirs.cache_dir();

    let build_output = Command::new("cross")
        .args([
            "build",
            "--target",
            "aarch64-unknown-linux-gnu",
            "-p",
            "runner",
        ])
        .current_dir(workspace_path)
        .output()
        .expect("Failed to run cargo build. Is Cargo installed and accessible to the application?");

    if !build_output.status.success() {
        return Err(WorkspaceError::BuildError(
            String::from_utf8(build_output.stderr)
                .unwrap_or_else(|_| "Could not parse cargo build output to utf8".to_owned()),
        )
        .into());
    }

    let runner_bin = fs::read(workspace_path.join("target/aarch64-unknown-linux-gnu/debug/runner"))
        .expect("Failed to read built runner binary, this might be an application logic error.");

    Ok(runner_bin)
}
