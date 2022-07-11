//! Functions used to manage the build workspace and build the runner with the provided probe-rs code
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::Result;
use axum::body::Bytes;
use cargo_toml::{Dependency, DependencyDetail, Manifest};
use fs_extra::dir::CopyOptions;
use tar::Archive;
use thiserror::Error;

use super::TaskRunnerError;

/// Path to the Hive workspace where the provided project is unpacked and built
const WORKSPACE_PATH: &str = "./data/workspace";
/// Path to where the built runner binary is stored
pub(super) const RUNNER_BINARY_PATH: &str = "./data/runner";
/// Path to the sourcefiles used to build the runner application
const RUNNER_SOURCE_PATH: &str = "./data/source";
const TESTCANDIDATE_SOURCE_PATH: &str = "./data/workspace/probe-rs-testcandidate";

/// Errors which happen if the provided probe-rs project is not correct
#[derive(Debug, Error)]
pub(super) enum WorkspaceError {
    #[error("No cargofile found in root folder")]
    NoWorkspaceCargoFile,
    #[error("No cargofile found in probe-rs folder")]
    NoProbeRsCargoFile,
    #[error("Crate probe-rs and its required dependencies not found in provided project")]
    WrongProject,
    #[error("Cargofile in root is not a workspace")]
    NoWorkspace,
    #[error("No hive directory with testfunctions found in provided probe-rs tests folder")]
    NoHiveDir,
}

/// Unpack the provided probe-rs tarball into the workspace and check if it is a valid probe-rs project.
/// If the checks succeed, the hive.rs file in the tests folder of the probe-rs project is copied into the runner for compilation.
///
/// # Panics
/// If the [`WORKSPACE_PATH`] does not exist. This means that the environment in which the monitor runs in has not been configured properly or if removing the cargofile of the provided tarball fails which is likely a permission issue.
pub(super) fn prepare_workspace(probe_rs_project: &Bytes) -> Result<()> {
    let workspace_path = Path::new(WORKSPACE_PATH);

    if !workspace_path.exists() {
        panic!("Could not find path {}. This is likely a configuration issue. Please make sure that the Hive workspace containing the sourcefiles is located at this path", WORKSPACE_PATH)
    }

    let project_path = Path::new(TESTCANDIDATE_SOURCE_PATH);

    let mut tarball = Archive::new(probe_rs_project.as_ref());

    tarball.unpack(project_path)?;

    check_and_remove_workspace_cargofile(&project_path)?;

    copy_hive_test_dir(&project_path, &workspace_path)?;

    modify_probe_rs_cargofile(&project_path)?;

    Ok(())
}

/// Checks the workspace Cargo.toml of the provided probe-rs project and deletes it to avoid nested workspaces which would fail the build process
///
/// This function checks whether the workspace contains a member named probe-rs. It fails if the workspace does not contain probe-rs
///
/// # Panics
/// In case any file system operations fail which indicate insufficient permissions or a corrupted install
fn check_and_remove_workspace_cargofile(project_path: &Path) -> Result<()> {
    let cargofile_path = project_path.join("Cargo.toml");

    if !cargofile_path.exists() {
        return Err(WorkspaceError::NoWorkspaceCargoFile.into());
    }

    let manifest = Manifest::from_path(&cargofile_path)?;

    if let Some(workspace) = manifest.workspace {
        if !workspace.members.contains(&"probe-rs".to_owned()) {
            return Err(WorkspaceError::WrongProject.into());
        }
    } else {
        return Err(WorkspaceError::NoWorkspace.into());
    }

    // The workspace cargofile has to be deleted, otherwise the build fails due to cargo discovering an unknown nested workspace
    fs::remove_file(cargofile_path).expect("Failed to delete workspace cargofile of probe-rs testcandidate. This is likely caused by insufficient permissions");

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
    fs_extra::copy_items(&[hive_path], runner_hive_path, &copy_options).expect("Failed to copy hive directory from probe-rs testcandidate to runner source files. This is likely due to a corrupted installation or missing permissions.");

    Ok(())
}

/// Modifies the probe_rs test crates cargofile hive-test dependency to depend on the local hive-test source on the testserver instead of any other source defined by the user.
///
/// This is required to avoid any circular dependencies or using outdated/incompatible versions on the testserver as well as allowing the build process to pass in case of custom path dependencies.
///
/// # Panics
/// In case any file system operations fail which indicate insufficient permissions or a corrupted install
fn modify_probe_rs_cargofile(project_path: &Path) -> Result<()> {
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

/// Restores the workspace to its defaults
///
/// # Panics
/// In case the function fails to read the contents of [`WORKSPACE_PATH`] which is caused by a corrupted installation or missing permissions.
/// Or if it fails to delete its contents or copy the runner source into the directory.
pub(super) fn restore_workspace() {
    let workspace_contents = fs::read_dir(WORKSPACE_PATH).expect("Failed to read Hive workspace directory. This might be caused by a corrupted installation of Hive or missing permissions.");

    for entry in workspace_contents.flatten() {
        let path = entry.path();
        if path.is_dir() {
            fs::remove_dir_all(path).expect("Failed to delete directory on workspace cleanup. Ensure that this function is only called when no part of the program is accessing it.");
        } else {
            fs::remove_file(path).expect("Failed to delete file on workspace cleanup. Ensure that this function is only called when no part of the program is accessing it.")
        }
    }

    let mut copy_options = CopyOptions::new();
    copy_options.overwrite = true;
    copy_options.copy_inside = true;

    let source_contents = fs::read_dir(RUNNER_SOURCE_PATH).expect("Failed to read Hive source directory. This might be caused by a corrupted installation of Hive or missing permissions.");
    let paths: Vec<PathBuf> = source_contents
        .flatten()
        .map(|entry| entry.path())
        .collect();

    fs_extra::copy_items(&paths, WORKSPACE_PATH, &copy_options).expect("Failed to copy runner source files into Hive workspace. This is likely due to a corrupted installation or missing permissions.");
}

/// Cleans the workspace after the build procedure.
/// It deletes the inserted probe-rs test build sourcefiles but leaves the runner sourcefiles in place
///
/// # Panics
/// If deleting the files fails which is usually caused by a corrupted installation or missing permissions
fn clean_workspace() {
    let testcandidate_contents = fs::read_dir(TESTCANDIDATE_SOURCE_PATH).expect("Failed to read Hive workspace testcandidate directory. This might be caused by a corrupted installation of Hive or missing permissions.");

    for entry in testcandidate_contents.flatten() {
        let path = entry.path();
        if path.is_dir() {
            fs::remove_dir_all(path).expect("Failed to delete directory on workspace cleanup. Ensure that this function is only called when no part of the program is accessing it.");
        } else {
            fs::remove_file(path).expect("Failed to delete file on workspace cleanup. Ensure that this function is only called when no part of the program is accessing it.")
        }
    }
}

/// Builds the runner binary with the provided probe-rs test dependency using Cargo
///
/// # Panics
/// If the [`RUNNER_BINARY_PATH`] does not exist. This means that the environment in which the monitor runs in has not been configured properly
pub(super) fn build_runner() -> Result<()> {
    if !Path::new(RUNNER_BINARY_PATH).exists() {
        panic!("Could not find path {}. This is likely a configuration issue. Please make sure that the ramdisk for storing the binary is correctly mounted at the requested path.", RUNNER_BINARY_PATH);
    }

    let build_output = Command::new("cargo")
        .args(["build", "-p", "runner", "--release"])
        .current_dir(WORKSPACE_PATH)
        .output()
        .expect("Failed to run cargo build. Is Cargo installed and accessible to the application?");

    if !build_output.status.success() {
        return Err(TaskRunnerError::BuildError(
            String::from_utf8(build_output.stderr)
                .unwrap_or_else(|_| "Could not parse cargo build output to utf8".to_owned()),
        )
        .into());
    }

    let mut copy_options = CopyOptions::new();
    copy_options.overwrite = true;

    // Copy runner from workspace to runner directory
    fs_extra::copy_items(&[&format!("{}/target/aarch64-unknown-linux-gnu/release/runner", WORKSPACE_PATH)], RUNNER_BINARY_PATH, &copy_options).expect("Failed to copy runner binary to runner directory. This is likely a configuration issue. Please make sure that the ramdisk for storing the runner binary is correctly mounted at the requested path.");

    clean_workspace();

    Ok(())
}
