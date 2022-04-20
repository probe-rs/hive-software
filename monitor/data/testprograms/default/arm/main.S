@ File: ARM thumb core independent default testprogram for Hive
@ Author: Thierry Kühni (TeyKey1)

.thumb
.syntax unified 
.cpu cortex-m0 @ use most limited arm cpu as base to ensure that the used instructions are valid for all cores

.include "hive_defines.S" @ include the Hive symbols

.text
    .global _start

@ setup vector table
vectortable:
    .word 0x0 @ stack pointer register (unused)
    .word _start + 1 @ reset handler
    .word 0x0 @ NMI
    .word _hardfaulthandler + 1
    .space 0xC8 @ leave enough space for mandatory vector table entries(length varies per cpu)

.thumb_func
_start:
    ldr r0, =HIVE_UID
    b loop

.thumb_func
_hardfaulthandler:
    b loop

.thumb_func
loop:
    nop
    b loop
