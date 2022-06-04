//! Handles the management of existing and new testprograms
use std::{fs::OpenOptions, io::Write, path::PathBuf};

use comm_types::hardware::Memory;
use serde::{Deserialize, Serialize};

mod defines;

pub(crate) const TESTPROGRAM_PATH: &str = "data/testprograms/";

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct TestProgram {
    pub name: String,
    pub path: PathBuf,
}

impl TestProgram {
    pub(crate) fn get_elf_path_arm(&self, memory_address: &Memory) -> PathBuf {
        self.path.join(format!(
            "arm/main_{:#x}_{:#x}.elf",
            memory_address.nvm.start, memory_address.ram.start
        ))
    }

    pub(crate) fn get_elf_path_riscv(&self, memory_address: &Memory) -> PathBuf {
        self.path.join(format!(
            "riscv/main_{:#x}_{:#x}.elf",
            memory_address.nvm.start, memory_address.ram.start
        ))
    }

    /// Inserts a newly generated `hive_defines.S` file into the provided testprogram
    ///
    /// # Panics
    /// If any file operation fails. This is usually caused by wrong/insufficient permissions or corrupted installs
    pub(crate) fn insert_hive_defines(&self) {
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
