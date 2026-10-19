.syntax unified
.arm
.fpu crypto-neon-fp-armv8
.hidden dn_gateway_enter
.hidden dn_gateway_leave
.hidden dn_gateway_tap

/* Save before executing any flag-changing instruction. The 328-byte frame
 * keeps native stack arguments at their original address and 8-byte alignment.
 * All d0-d31 are saved because taps can occur inside hard-VFP native routines.
 */
.macro SAVE
    push {r0-r12, lr}
    vpush {d0-d15}
    vpush {d16-d31}
    mrs r2, cpsr
    vmrs r3, fpscr
    push {r2, r3}
    sub sp, sp, #8
.endm
.macro RESTORE
    str r0, [sp]
    ldr r2, [sp, #8]
    ldr r3, [sp, #12]
    /* GE is dead at these eleven audited hook sites; NZCVQ-only restoration
     * avoids Ryujinx's unsupported APSR_g translation. See GE_PRESERVATION_AUDIT.md.
     * This leaves callback GE untouched; it neither restores nor clears it. */
    msr APSR_nzcvq, r2
    vmsr fpscr, r3
    add sp, sp, #16
    vpop {d16-d31}
    vpop {d0-d15}
    pop {r0-r12, lr}
    ldr pc, [sp, #-328]
.endm
.macro ENTRY id, helper
    .section .text.dn_gateway_\id,"ax",%progbits
    .balign 4
    .global dn_gateway_\id
    .hidden dn_gateway_\id
    .type dn_gateway_\id,%function
dn_gateway_\id:
    SAVE
    mov r0, #\id
    mov r1, sp
    bl \helper
    RESTORE
    .size dn_gateway_\id, .-dn_gateway_\id
.endm
ENTRY 0, dn_gateway_enter
ENTRY 1, dn_gateway_enter
ENTRY 2, dn_gateway_enter
ENTRY 3, dn_gateway_enter
ENTRY 4, dn_gateway_tap
ENTRY 5, dn_gateway_tap
/* Production melee and shot HZV entries, also exercised by the ABI fixture. */
ENTRY 6, dn_gateway_enter
ENTRY 7, dn_gateway_enter
/* Status damage: poison tick entry, blast entry tap and mount-finisher tap. */
ENTRY 8, dn_gateway_enter
ENTRY 9, dn_gateway_tap
ENTRY 10, dn_gateway_tap

.section .text.dn_gateway_after,"ax",%progbits
.balign 4
.global dn_gateway_after
.hidden dn_gateway_after
.type dn_gateway_after,%function
dn_gateway_after:
    SAVE
    mov r0, sp
    bl dn_gateway_leave
    RESTORE
.size dn_gateway_after, .-dn_gateway_after
.section .note.GNU-stack,"",%progbits
