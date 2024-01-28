//! Handles the management of existing and new testprograms
//!
//! Testprograms are programs which can be flashed onto the targets of the testrack prior to testing. The flashed testprograms are then used as a means to test the probe-rs testcandidate.
//! The monitor can have multiple testprograms. However, only one can be active at a time and be used for testing. Currently only assembly is supported as language to write testprograms.
//!
//! ## Default Testprogram
//! The default testprogram is a special kind of testprogram as it is included with every Hive installation and cannot be modified by the user. It acts as the default in case building of any other selected active testprogram fails.
//!
//! # Hive Defines
//! Hive defines are variables which can be injected into a testprogram at build time. In general before a testprogram is built the hive_defines.S file is generated which contains the
//! generated variables that can be imported and used by the testprogram.
//!
//! A popular example of what can be done using Hive Defines is a unique ID per testprogram build which then can be tested against in the testfunction.
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use async_graphql::{Enum, SimpleObject};
use comm_types::hardware::Memory;
use controller::hardware::HiveHardware;
use hive_db::BincodeTransactional;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use sled::transaction::UnabortableTransactionError;

use crate::database::{keys, MonitorDb};

use self::build::BuildError;

mod address;
mod build;
pub mod defines;

/// Path to where the testprograms are stored
pub const TESTPROGRAM_PATH: &str = "data/testprograms/";
/// The name of the default testprogram
pub const DEFAULT_TESTPROGRAM_NAME: &str = "default";
/// A temporary workspace for the assembler output
const ASSEMBLER_TEMP_WORKSPACE_PATH: &str = "data/assembler_workspace";

lazy_static! {
    static ref ASSEMBLER_WORKSPACE_LOCK: Mutex<()> = Mutex::new(());
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Enum)]
pub enum TestprogramStatus {
    NotInitialized,
    CompileFailure,
    Ok,
}

impl TestprogramStatus {
    /// Check if status is [`TestprogramStatus::Ok`]
    fn is_ready(&self) -> bool {
        *self == TestprogramStatus::Ok
    }
}

/// The current supported architectures of a Testprogram
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Enum)]
pub enum Architecture {
    Arm,
    Riscv,
}

/// Like [`Memory`] but only contains start addresses of the range
pub struct MemoryStart {
    pub nvm: u64,
    pub ram: u64,
}

impl From<&Memory> for MemoryStart {
    fn from(memory: &Memory) -> Self {
        Self {
            nvm: memory.nvm.start,
            ram: memory.ram.start,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, SimpleObject)]
pub struct Testprogram {
    name: String,
    #[graphql(skip)]
    path: PathBuf,
    testprogram_arm: TestprogramArchitecture,
    testprogram_riscv: TestprogramArchitecture,
}

impl Testprogram {
    /// Create a new testprogram
    ///
    /// # Panics
    /// If the new testprogram directory cannot be created which is either caused by missing permissions or a corrputed install
    pub fn new(name: String) -> Self {
        let path = PathBuf::from_str(TESTPROGRAM_PATH).unwrap().join(&name);

        fs::create_dir_all(&path).expect("Failed to create directory for Testprogram");

        // create blank main.S File
        fs::create_dir(path.join("arm")).expect("Failed to create ARM testprogram directory");
        fs::create_dir(path.join("riscv")).expect("Failed to create RISCV testprogram directory");

        OpenOptions::new()
            .write(true)
            .create(true)
            .open(path.join("arm/main.S"))
            .expect("Failed to create ARM main.S File in testprogram");
        OpenOptions::new()
            .write(true)
            .create(true)
            .open(path.join("riscv/main.S"))
            .expect("Failed to create RISCV main.S File in testprogram");

        Self {
            name: name.clone(),
            path,
            testprogram_arm: TestprogramArchitecture::new(name.clone(), Architecture::Arm),
            testprogram_riscv: TestprogramArchitecture::new(name, Architecture::Riscv),
        }
    }

    /// Create the default testprogram
    pub fn create_default() -> Self {
        let path = PathBuf::from_str(TESTPROGRAM_PATH)
            .unwrap()
            .join(DEFAULT_TESTPROGRAM_NAME);

        let mut testprogram_arm =
            TestprogramArchitecture::new(DEFAULT_TESTPROGRAM_NAME.to_owned(), Architecture::Arm);
        let mut testprogram_riscv =
            TestprogramArchitecture::new(DEFAULT_TESTPROGRAM_NAME.to_owned(), Architecture::Riscv);

        testprogram_arm.set_status(TestprogramStatus::Ok);
        testprogram_riscv.set_status(TestprogramStatus::Ok);

        Self {
            name: DEFAULT_TESTPROGRAM_NAME.to_owned(),
            path,
            testprogram_arm,
            testprogram_riscv,
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_path(&self) -> &Path {
        &self.path
    }

    /// If the testprogram is ready to be assembled and linked
    pub fn is_ready(&self) -> bool {
        self.testprogram_arm.status.is_ready() && self.testprogram_riscv.status.is_ready()
    }

    /// Builds binaries for each architecture after inserting the current hive defines. Automatically builds multiple binaries with different flash/ram start addresses according to the needs of the currently connected targets.
    pub fn build_binaries(&mut self, hardware: &HiveHardware) -> Result<(), BuildError> {
        let addresses = address::get_and_init_target_address_ranges(hardware);

        self.insert_hive_defines();

        self.testprogram_arm.assemble_binary()?;
        self.testprogram_riscv.assemble_binary()?;

        let mut arm_start_addresses: Vec<MemoryStart> = vec![];

        addresses.arm.iter().for_each(|address| {
            if !arm_start_addresses.iter().any(|start_address| {
                address.nvm.start == start_address.nvm && address.ram.start == start_address.ram
            }) {
                arm_start_addresses.push(address.into());
            }
        });

        let mut riscv_start_addresses: Vec<MemoryStart> = vec![];

        addresses.riscv.iter().for_each(|address| {
            if !riscv_start_addresses.iter().any(|start_address| {
                address.nvm.start == start_address.nvm && address.ram.start == start_address.ram
            }) {
                riscv_start_addresses.push(address.into());
            }
        });

        for arm_start_address in arm_start_addresses.iter() {
            self.testprogram_arm.link_binary(arm_start_address)?;
        }

        for riscv_start_address in riscv_start_addresses.iter() {
            self.testprogram_riscv.link_binary(riscv_start_address)?;
        }

        Ok(())
    }

    pub fn get_arm(&self) -> &TestprogramArchitecture {
        &self.testprogram_arm
    }

    pub fn get_arm_mut(&mut self) -> &mut TestprogramArchitecture {
        &mut self.testprogram_arm
    }

    pub fn get_riscv(&self) -> &TestprogramArchitecture {
        &self.testprogram_riscv
    }

    pub fn get_riscv_mut(&mut self) -> &mut TestprogramArchitecture {
        &mut self.testprogram_riscv
    }

    /// Inserts a newly generated `hive_defines.S` file into the provided testprogram
    ///
    /// # Panics
    /// If any file operation fails. This is usually caused by wrong/insufficient permissions or corrupted installs
    fn insert_hive_defines(&self) {
        log::debug!(
            "Inserting the hive_defines.S files into the '{}' testprogram",
            self.name
        );
        let open_options = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .to_owned();

        let mut arm_defines = open_options.open(self.path.to_owned().join("arm/hive_defines.S")).expect("Failed to open/create the hive_defines.S files. This might be caused by insufficient permissions.");
        let mut riscv_defines = open_options.open(self.path.to_owned().join("riscv/hive_defines.S")).expect("Failed to open/create the hive_defines.S files. This might be caused by insufficient permissions.");

        let content = defines::generate_defines();

        arm_defines
            .write_all(content.as_bytes())
            .expect("Failed to write the hive_defines.S file contents.");
        riscv_defines
            .write_all(content.as_bytes())
            .expect("Failed to write the hive_defines.S file contents.");
    }
}

/// The sub-instance of [`Testprogram`] which contains all architecture specific functionality
#[derive(Debug, Serialize, Deserialize, SimpleObject)]
pub struct TestprogramArchitecture {
    #[graphql(skip)]
    path: PathBuf,
    architecture: Architecture,
    status: TestprogramStatus,
    compile_message: String,
}

impl TestprogramArchitecture {
    pub fn new(name: String, architecture: Architecture) -> Self {
        let folder = match architecture {
            Architecture::Arm => "arm",
            Architecture::Riscv => "riscv",
        };

        Self {
            path: PathBuf::from_str(TESTPROGRAM_PATH)
                .unwrap()
                .join(name)
                .join(folder),
            architecture,
            status: TestprogramStatus::NotInitialized,
            compile_message: String::new(),
        }
    }

    /// Manually set the status of this testprogram
    pub fn set_status(&mut self, status: TestprogramStatus) {
        self.status = status;
    }

    /// Get the path to the elf file with the provided memory address range
    pub fn get_elf_path(&self, memory_address: &Memory) -> PathBuf {
        self.path.join(format!(
            "main_{:#x}_{:#x}.elf",
            memory_address.nvm.start, memory_address.ram.start
        ))
    }

    /// Checks the provided source code by trying to assemble it. If assembly succeeds the source code is saved to disk.
    ///
    /// The status and compile message of the testprogram are set according to the assembler output and status.
    ///
    /// # Panics
    /// If any file operation fails. This is usually caused by wrong/insufficient permissions or corrupted installs
    pub fn check_source_code(&mut self, source_code: &[u8]) {
        // Aquire lock to use the assembler workspace
        let lock = ASSEMBLER_WORKSPACE_LOCK.lock().unwrap();

        let mut assembler = match self.architecture {
            Architecture::Arm => Command::new("arm-none-eabi-as").stdin(Stdio::piped())
                .current_dir(ASSEMBLER_TEMP_WORKSPACE_PATH)
                .args(["-mthumb"])
                .spawn()
                .expect("Failed to run the ARM assembly process, is the arm-none-eabi-as command accessible to the application?"),
            Architecture::Riscv => Command::new("riscv-none-elf-as").stdin(Stdio::piped())
                .current_dir(ASSEMBLER_TEMP_WORKSPACE_PATH)
                .spawn()
                .expect("Failed to run the RISCV assembly process, is the riscv-none-elf-as command accessible to the application?"),
        };

        assembler
            .stdin
            .as_mut()
            .unwrap()
            .write_all(source_code)
            .unwrap();

        let output = assembler.wait_with_output().unwrap();

        let workspace_contents = fs::read_dir(ASSEMBLER_TEMP_WORKSPACE_PATH).expect("Failed to read Hive assembler workspace directory. This might be caused by a corrupted installation of Hive or missing permissions.");

        for entry in workspace_contents.flatten() {
            let path = entry.path();
            if path.is_dir() {
                fs::remove_dir_all(path).expect("Failed to delete directory on workspace cleanup. Ensure that this function is only called when no part of the program is accessing it.");
            } else {
                fs::remove_file(path).expect("Failed to delete file on workspace cleanup. Ensure that this function is only called when no part of the program is accessing it.")
            }
        }

        drop(lock);

        if output.status.success() {
            let open_options = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .to_owned();

            let mut code_file = open_options
                .open(self.path.to_owned().join("main.S"))
                .unwrap();

            code_file.write_all(source_code).unwrap();

            self.status = TestprogramStatus::Ok;
            self.compile_message = String::from_utf8(output.stdout).unwrap_or_else(|err| {
                format!("Failed to transform assembler output to string: {}", err)
            });
        } else {
            self.status = TestprogramStatus::CompileFailure;
            self.compile_message = String::from_utf8(output.stdout).unwrap_or_else(|err| {
                format!("Failed to transform assembler output to string: {}", err)
            });
        }
    }

    /// Assemble the binary and store the resulting objectfile on disk
    ///
    /// If assembly fails the status of this testprogram is set to [`TestprogramStatus::CompileFailure`] and the error forwarded
    pub fn assemble_binary(&mut self) -> Result<(), BuildError> {
        let assemble_result = match self.architecture {
            Architecture::Arm => build::assemble_binary_arm(&self.path),
            Architecture::Riscv => build::assemble_binary_riscv(&self.path),
        };

        if let Err(err) = assemble_result {
            log::warn!("{}", err);
            self.compile_message = err.to_string();
            self.status = TestprogramStatus::CompileFailure;

            Err(err)
        } else {
            Ok(())
        }
    }

    /// Links the binary according to the provided memory address range and saves the resulting elf file to disk
    ///
    /// If assembly fails the status of this testprogram is set to [`TestprogramStatus::CompileFailure`] and the error forwarded
    pub fn link_binary(&mut self, address_range: &MemoryStart) -> Result<(), BuildError> {
        let link_result = match self.architecture {
            Architecture::Arm => build::link_binary_arm(&self.path, address_range),
            Architecture::Riscv => build::link_binary_riscv(&self.path, address_range),
        };

        if let Err(err) = link_result {
            log::warn!("{}", err);
            self.compile_message = err.to_string();
            self.status = TestprogramStatus::CompileFailure;

            Err(err)
        } else {
            Ok(())
        }
    }
}

/// Rebuilds and links the binaries for the currently available targets on the Testrack. In case the building or linking of the currently active testprogram fails, the function falls back to the default testprogram.
///
/// # Panics
/// In case building or linking fails on the default testprogram.
pub fn sync_binaries(db: Arc<MonitorDb>, hardware: &HiveHardware) {
    let mut active_testprogram = db.config_tree.transaction::<_, _, UnabortableTransactionError>(|tree|{
        let active = tree.b_get(&keys::config::ACTIVE_TESTPROGRAM)?.expect("Failed to get the active testprogram. Flashing the testbinaries can only be performed once the active testprogram is known");

        let mut testprograms = tree.b_get(&keys::config::TESTPROGRAMS)?.expect("DB not initialized");

        for idx in 0..testprograms.len() {
            if active != testprograms[idx].get_name() {
                continue;
            }

            return Ok(testprograms.remove(idx));
        }
        panic!("Failed to find active testprogram in database. This should not happen as it indicates a desync between the active testprogram DB data and the testprogram DB data.");
    }).unwrap();

    if active_testprogram.is_ready() && active_testprogram.build_binaries(hardware).is_ok() {
        return;
    }

    log::warn!("Failed to build or link the currently active testprogram '{}', falling back to default testprogram.", active_testprogram.get_name());

    // Set active testprogram to default and retry
    let mut active_testprogram = db.config_tree.transaction::<_, _, UnabortableTransactionError>(|tree|{
        tree.b_insert(&keys::config::ACTIVE_TESTPROGRAM, &DEFAULT_TESTPROGRAM_NAME.to_owned())?.expect("Failed to get the active testprogram. Flashing the testbinaries can only be performed once the active testprogram is known");

        let mut testprograms = tree.b_get(&keys::config::TESTPROGRAMS)?.expect("DB not initialized");

        for idx in 0..testprograms.len() {
            if DEFAULT_TESTPROGRAM_NAME != testprograms[idx].get_name() {
                continue;
            }

            return Ok(testprograms.remove(idx));
        }
        panic!("Failed to find default testprogram in database. This should not happen as the default testprogram may not be deleted or modified.");
    }).unwrap();

    active_testprogram
        .build_binaries(hardware)
        .expect("Failed to build or link default testprogram.")
}
