# Default Hive Testprogram
This is the default Hive testprogram and included/mandatory in every install of Hive. 

# Functionality
This program loads the HIVE_UID into the first CPU register (ARM: r0, RISCV: x10) and then jumps into an infinite nop loop.

This program can be used to verify if in fact the correct new program has been flashed or if the old one is still running and flashing failed silently for some reason.