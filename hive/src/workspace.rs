//! Functions used to manage the build workspace and build the runner with the provided probe-rs code
//!
//! In order to build the runner binary the application needs a workspace directory. The workspace directory is chosen dynamically according to the OS the binary is executed on.
//! Generally the directory is created in some application cache related folder.
//!
//! To build the runner the runner source as well as the probe-rs testcandidate is required. The probe-rs testcandidate is copied into the workspace from the local FS path which is provided by the user.
//! The runner source code is cloned/pulled from the [`RUNNER_SOURCE_REPO`] repository using libgit2. The workspace ensures that the correct source code version
//! is pulled from the repository which is indicated by the [`REPO_REFERENCE`].
//!
//! The cli performs some manipulation on the source code in the workspace in order to prepare it for building, such as removing the probe-rs testcandidate workspace to avoid having unknown nested workspaces.
//!
//! After building the workspace is cleaned by removing the probe-rs testcandidate source but keeping the runner source and cargo/cross build cache in place to allow for a faster build process on the next test request
use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::{anyhow, bail, Result};
use fs_extra::dir::CopyOptions;
use git2::build::{CheckoutBuilder, RepoBuilder};
use git2::Error as GitError;
use git2::Repository;
use ignore::WalkBuilder;
use thiserror::Error;
use toml::{Table, Value};

use super::config;

/// Link to the runner source repository
const RUNNER_SOURCE_REPO: &str = "https://github.com/probe-rs/hive-software.git";
/// The git reference which should be used as runner source version. Can be a tag or branch for example
const REPO_REFERENCE: &str = "refs/remotes/origin/master";

/// Errors which happen if the provided probe-rs project is not correct
#[derive(Debug, Error)]
pub enum WorkspaceError {
    #[error("No cargofile found in probe-rs folder")]
    NoProbeRsCargoFile,
    #[error("No hive directory with testfunctions found in provided probe-rs tests folder")]
    NoHiveDir,
    #[error("An error related to git occurred.")]
    GitError(#[from] GitError),
    #[error("Failed to build the test runner.\n\nCargo build output:\n{0}")]
    BuildError(String),
}

/// Unpack the provided probe-rs tarball into the workspace and check if it is a valid probe-rs project.
/// If the checks succeed, the hive.rs file in the tests folder of the probe-rs project is copied into the runner for compilation.
///
/// # Panics
/// If any FS related call fails.
pub fn prepare_workspace(probe_rs_project: &Path) -> Result<()> {
    let project_dirs = config::get_project_dirs();
    let workspace_path = project_dirs.cache_dir();

    let testcandidate_path = workspace_path.join("probe-rs-hive-testcandidate");
    let hive_software_path = workspace_path.join("hive-software");

    if !workspace_path.exists() {
        fs::create_dir_all(workspace_path).expect("Failed to create cache directory for Hive CLI");
    }

    if !testcandidate_path.exists() {
        fs::create_dir_all(&testcandidate_path)
            .expect("Failed to create testcandidate cache directory for Hive CLI");
    }

    prepare_runner_source(&hive_software_path)?;

    copy_testcandidate_source_to_workspace(probe_rs_project, &testcandidate_path)?;

    modify_testcandidate_probe_rs_cargofile(&testcandidate_path)?;

    copy_hive_test_dir(&testcandidate_path, &hive_software_path)?;

    Ok(())
}

/// Clone the source code of the runner into the workspace if the workspace does not exist yet.
/// If a repository is already detected it checks if it matches the required verioning tag and updates if required.
///
/// # Panics
/// If the requested [`REPO_REFERENCE`] does not exist in the repository
fn prepare_runner_source(hive_software_path: &Path) -> Result<(), WorkspaceError> {
    match Repository::open(hive_software_path) {
        Ok(repo) => {
            let mut remote = repo.find_remote("origin").expect("Failed to find remote 'origin' in local workspace repository. This might be a corrupted installation.");

            remote.fetch(&["master"], None, None)?;

            let reference = repo
                .revparse_single(REPO_REFERENCE)
                .expect("Failed to find repository reference specified in binary. This should not happen, please file an issue.");
            let mut checkout_builder = CheckoutBuilder::new();
            checkout_builder.force().target_dir(hive_software_path);

            repo.checkout_tree(&reference, Some(&mut checkout_builder))?;
            repo.set_head(REPO_REFERENCE)?;
        }
        Err(_) => {
            log::info!(
                "Runner source repository was not found in workspace, cloning specified version..."
            );

            let mut checkout_builder = CheckoutBuilder::new();
            checkout_builder.force().target_dir(hive_software_path);

            RepoBuilder::new()
                .with_checkout(checkout_builder)
                .branch(REPO_REFERENCE.split('/').last().unwrap())
                .clone(RUNNER_SOURCE_REPO, hive_software_path)?;
        }
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

    for result in WalkBuilder::new(probe_rs_project)
        .require_git(false)
        .build()
    {
        match result {
            Ok(entry) => {
                let relative_path = pathdiff::diff_paths(entry.path(), probe_rs_project).unwrap();

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

/// Modifies the probe_rs testcandidate crates cargofile hive-test dependency to depend on the local hive-test source on the testserver instead of any other source defined by the user.
///
/// This is required to avoid any circular dependencies or using outdated/incompatible versions on the testserver as well as allowing the build process to pass in case of custom path dependencies.
///
/// # Panics
/// In case any file system operations fail which indicate insufficient permissions or a corrupted install
fn modify_testcandidate_probe_rs_cargofile(testcandidate_path: &Path) -> Result<()> {
    let cargofile_path = testcandidate_path.join("probe-rs/Cargo.toml");

    if !cargofile_path.exists() {
        return Err(WorkspaceError::NoProbeRsCargoFile.into());
    }

    let manifest_content = fs::read_to_string(&cargofile_path)?;
    let mut manifest = manifest_content.parse::<Table>()?;

    let dev_dependencies = manifest.get_mut("dev-dependencies").ok_or(anyhow!(
        "Failed to find dev-dependencies in probe-rs Cargo.toml file"
    ))?;

    if let Value::Table(deps) = dev_dependencies {
        let hive_test = deps.get_mut("hive-test").ok_or(anyhow!("Failed to find hive-test dependency in probe-rs Cargo.toml file. Have you properly set up your Hive tests inside your probe-rs project?"))?;

        let mut hive_test_dependency = Table::new();
        hive_test_dependency.insert(
            "path".to_owned(),
            Value::String(String::from("../../hive-software/hive-test")),
        );

        *hive_test = Value::Table(hive_test_dependency);

        fs::write(cargofile_path, toml::to_string_pretty(&manifest).expect("Failed to serialize newly formed Cargo.toml data. This is a bug, please open an issue and provide your probe-rs Cargo.toml file.")).expect("Failed to write modified probe-rs Cargo.toml file. This is likely due to a corrupted installation or missing permissions.");
    }

    /*
    TODO: this code fails with cargo_toml 0.19 due to ValueAfterTable error workaround is to just use toml for now

    let mut manifest = Manifest::from_path(&cargofile_path)?;

    if let Some(hive_test) = manifest.dev_dependencies.get_mut("hive-test") {
        let hive_test_dependency = DependencyDetail {
            package: Some("hive-test".to_owned()),
            path: Some(String::from("../../hive-software/hive-test")),
            ..Default::default()
        };

        *hive_test = Dependency::Detailed(Box::new(hive_test_dependency));

        fs::write(cargofile_path, toml::to_string_pretty(&manifest).unwrap()).expect("Failed to write modified probe-rs Cargo.toml file. This is likely due to a corrupted installation or missing permissions.");
    } else {
        bail!("Failed to find hive-test dependency in probe-rs Cargo.toml file. Have you properly set up your Hive tests inside your probe-rs project?")
    }*/

    Ok(())
}

/// Copy the hive test directory of probe-rs tests into the runner source
///
/// # Panics
/// In case any file system operations fail which indicate insufficient permissions or a corrupted install
fn copy_hive_test_dir(testcandidate_path: &Path, hive_software_path: &Path) -> Result<()> {
    let hive_path = testcandidate_path.join("probe-rs/tests/hive/");

    if !hive_path.exists() {
        return Err(WorkspaceError::NoHiveDir.into());
    }

    let mut copy_options = CopyOptions::new();
    copy_options.overwrite = true;
    copy_options.copy_inside = true;

    let runner_hive_path = hive_software_path.join("runner/src/");
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

    let testcandidate_path = workspace_path.join("probe-rs-hive-testcandidate");

    if !testcandidate_path.exists() {
        // Nothing needs cleaning
        return;
    }

    let testcandidate_contents = fs::read_dir(testcandidate_path).expect(
        "Failed to read probe-rs-hive-testcandidate directory. This might be caused by missing permissions.",
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
    let hive_software_path = workspace_path.join("hive-software");

    let build_output = Command::new("cross")
        .args([
            "build",
            "--manifest-path",
            "./hive-software/Cargo.toml",
            "--target",
            "aarch64-unknown-linux-gnu",
            "-p",
            "runner",
        ])
        .current_dir(workspace_path)
        .output()
        .expect("Failed to run cross build. Is Cross installed and accessible to the application?");

    if !build_output.status.success() {
        return Err(WorkspaceError::BuildError(
            String::from_utf8(build_output.stderr)
                .unwrap_or_else(|_| "Could not parse cargo build output to utf8".to_owned()),
        )
        .into());
    }

    let runner_bin = fs::read(
        hive_software_path.join("target/aarch64-unknown-linux-gnu/debug/runner"),
    )
    .expect("Failed to read built runner binary, this might be an application logic error.");

    Ok(runner_bin)
}
