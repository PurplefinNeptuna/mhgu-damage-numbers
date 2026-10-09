/* Native leaf/recursive/collision fixtures used by the gateway probes.
 * Leaf arithmetic deliberately changes flags and VFP values to test restoration. */

.syntax unified
.arm
.fpu crypto-neon-fp-armv8
.section .text.dn_probe_leaf,"ax",%progbits
.balign 4
.global dn_probe_leaf
.type dn_probe_leaf,%function
dn_probe_leaf:
    ldr r0, [sp]
    add r1, r1, r0
    adds r2, r2, r3
    vadd.f32 s0, s0, s1
    veor q8, q8, q9
    vmrs r3, fpscr
    bx lr
.size dn_probe_leaf, .-dn_probe_leaf

.section .text.dn_probe_recursive,"ax",%progbits
.balign 4
.global dn_probe_recursive
.type dn_probe_recursive,%function
dn_probe_recursive:
    push {r4, lr}
    cmp r0, #0
    beq 1f
    mov r4, r0
    sub r0, r0, #1
    bl dn_probe_recursive
    add r0, r0, r4
1:  pop {r4, pc}
.size dn_probe_recursive, .-dn_probe_recursive

/* Explicit collision-body fixture after the game's displaced entry words.
 * This routes to the original HZV and damage routines; it does not claim to
 * execute the real collision traversal or combined raw/element calculation.
 */
.section .text.dn_probe_collision_body,"ax",%progbits
.balign 4
.global dn_probe_collision_body
.type dn_probe_collision_body,%function
dn_probe_collision_body:
    mov r4, r1
    mov r5, r2
    mov r0, #0
    mov r1, r5
    ldr r2, =0x30092000
    ldr r3, =0x30093000
    mov r12, #0
    str r12, [sp]
    mov r12, #3
    str r12, [sp, #4]
    vldr s0, 1f
    vldr s1, 2f
    ldr r12, =0x0416dc6c
    blx r12
    mov r0, r5
    mov r1, #3
    ldrsh r2, [r4, #0x38]
    mov r3, #1
    ldr r12, =0x0409d6f8
    blx r12
    add sp, sp, #16
    pop {r4-r8, pc}
1:  .float 100.0
2:  .float 0.0
.ltorg
.size dn_probe_collision_body, .-dn_probe_collision_body

.section .text.dn_probe_collision_shot_body,"ax",%progbits
.balign 4
.global dn_probe_collision_shot_body
.type dn_probe_collision_shot_body,%function
dn_probe_collision_shot_body:
    mov r4, r1
    mov r5, r2
    mov r0, #0
    ldr r3, =0x30093020
    mov r12, #0
    str r12, [sp]
    ldr r12, =0x30093008
    str r12, [sp, #4]
    ldr r12, =0x3009300c
    str r12, [sp, #8]
    ldr r12, =0x04178660
    blx r12
    mov r0, r5
    mov r1, #3
    ldrsh r2, [r4, #0x38]
    mov r3, #1
    ldr r12, =0x0409d6f8
    blx r12
    add sp, sp, #16
    pop {r4-r8, pc}
.ltorg
.size dn_probe_collision_shot_body, .-dn_probe_collision_shot_body
.section .note.GNU-stack,"",%progbits
