# File: riscv rv32i core independent test binary
# Author: Thierry Kühni

.include "hive_defines.S"

.text
    .global _start

_start:
    li x10, HIVE_UID
    j loop

loop:
    nop
    j loop
