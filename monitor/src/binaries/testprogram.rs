//! Handles the management of existing and new testprograms
use std::path::PathBuf;

use comm_types::hardware::Memory;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct TestProgram {
    pub name: String,
    pub path: PathBuf,
}

impl TestProgram {
    pub(super) fn get_elf_path_arm(&self, memory_address: &Memory) -> PathBuf {
        self.path.join(format!(
            "arm/main_{}_{}.elf",
            memory_address.nvm.start, memory_address.ram.start
        ))
    }

    pub(super) fn get_elf_path_riscv(&self, memory_address: &Memory) -> PathBuf {
        self.path.join(format!(
            "riscv/main_{}_{}.elf",
            memory_address.nvm.start, memory_address.ram.start
        ))
    }
}

pub(crate) const TESTPROGRAM_PATH: &str = "data/testprograms/";

fn add_testprogram() {
    todo!("");
}
