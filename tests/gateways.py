#!/usr/bin/env python3
"""Execute the production NRO's nine ARM gateways with deliberately clobbering callback doubles.

This exercises compiled registers/stack/return paths; it does not emulate Forge or gameplay.
GE is deliberately excluded from equality because it is dead at these eight audited continuations.
"""
import json,struct,subprocess,sys
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'scripts'))
from paths import MOD,BUILD_ROOT,CARGO_TARGET,CONFIG
import unicorn
from unicorn import arm_const as arm
MODULE,ENGINE,RETURN,STACK=0x31000000,0x32000000,0x32000100,0x41010000
elf=CARGO_TARGET/'armv7a-none-eabi/release/libdamage_numbers_rs.so'
nm=Path(CONFIG['devkitarm'])/'bin/arm-none-eabi-nm'
names={}
for line in subprocess.check_output([nm,'-n',elf],text=True).splitlines():
 parts=line.split()
 if len(parts)==3 and parts[2].startswith('dn_gateway_'):names[parts[2]]=MODULE+int(parts[0],16)
data=(BUILD_ROOT/'package/romfs/nativeNX/plugins/damage_numbers_rs.nro').read_bytes()
regs=[getattr(arm,'UC_ARM_REG_R'+str(i))for i in range(13)]
ds=[getattr(arm,'UC_ARM_REG_D'+str(i))for i in range(32)]
checks=0
for site in range(8):
 for ge in range(16):
  for change_ge in [False,True]:
   u=unicorn.Uc(unicorn.UC_ARCH_ARM,unicorn.UC_MODE_ARM);u.ctl_set_cpu_model(arm.UC_CPU_ARM_MAX)
   u.mem_map(MODULE,((len(data)+4095)//4096+64)*4096);u.mem_write(MODULE,data)
   u.mem_map(ENGINE,4096);u.mem_write(ENGINE,struct.pack('<I',0xe12fff1e));u.mem_map(0x41000000,0x20000)
   u.reg_write(arm.UC_ARM_REG_C1_C0_2,0xf00000);u.reg_write(arm.UC_ARM_REG_FPEXC,0x40000000)
   u.reg_write(arm.UC_ARM_REG_CPSR,0xa8000013|(ge<<16));u.reg_write(arm.UC_ARM_REG_FPSCR,0xa1000000)
   for i,r in enumerate(regs):u.reg_write(r,0x11220000+i*257)
   for i,r in enumerate(ds):u.reg_write(r,0x40010000abcd0000+i)
   u.reg_write(arm.UC_ARM_REG_SP,STACK);u.reg_write(arm.UC_ARM_REG_LR,RETURN)
   stack=bytes(range(64));u.mem_write(STACK,stack)
   initial=[u.reg_read(r)for r in regs];initial_d=[u.reg_read(r)for r in ds];fpscr=u.reg_read(arm.UC_ARM_REG_FPSCR);flags=u.reg_read(arm.UC_ARM_REG_CPSR)&0xf8000000
   callbacks=[]
   def trace(cpu,address,size,_):
    if address==RETURN:cpu.emu_stop();return
    entering=address in [names['dn_gateway_enter'],names['dn_gateway_tap']]
    leaving=address==names['dn_gateway_leave']
    if not entering and not leaving:return
    ptr=cpu.reg_read(arm.UC_ARM_REG_R1 if entering else arm.UC_ARM_REG_R0);assert ptr==STACK-328 and ptr%8==0
    raw=bytes(cpu.mem_read(ptr,328));assert list(struct.unpack_from('<13I',raw,272))==initial
    assert list(struct.unpack_from('<16Q',raw,144))+list(struct.unpack_from('<16Q',raw,16))==initial_d
    assert struct.unpack_from('<I',raw,12)[0]==fpscr
    assert struct.unpack_from('<I',raw,8)[0]&0xf8000000==flags
    callbacks.append('entry'if entering else 'return')
    around=site not in [4,5]
    if entering:assert cpu.reg_read(arm.UC_ARM_REG_R0)==site
    if around:cpu.mem_write(ptr+324,struct.pack('<I',names['dn_gateway_after']if entering else RETURN))
    ret=cpu.reg_read(arm.UC_ARM_REG_LR)
    for i,r in enumerate(regs+[arm.UC_ARM_REG_LR]):cpu.reg_write(r,0xdead0000+i)
    for i,r in enumerate(ds):cpu.reg_write(r,0x7ff10000ffff0000+i)
    cpu.reg_write(arm.UC_ARM_REG_FPSCR,0x50000000)
    cpsr=cpu.reg_read(arm.UC_ARM_REG_CPSR);cpu.reg_write(arm.UC_ARM_REG_CPSR,(cpsr&~0xf80f0000)|0x58000000|((15-ge if change_ge else ge)<<16))
    cpu.reg_write(arm.UC_ARM_REG_R0,ENGINE if entering else RETURN);cpu.reg_write(arm.UC_ARM_REG_PC,ret)
   u.hook_add(unicorn.UC_HOOK_CODE,trace);u.emu_start(names['dn_gateway_'+str(site)],RETURN+4,count=500)
   assert [u.reg_read(r)for r in regs]==initial
   assert [u.reg_read(r)for r in ds]==initial_d
   assert u.reg_read(arm.UC_ARM_REG_FPSCR)==fpscr
   assert u.reg_read(arm.UC_ARM_REG_CPSR)&0xf8000000==flags
   assert u.reg_read(arm.UC_ARM_REG_CPSR)&0xf0000==(15-ge if change_ge else ge)<<16
   assert u.reg_read(arm.UC_ARM_REG_SP)==STACK and u.reg_read(arm.UC_ARM_REG_LR)==RETURN
   assert bytes(u.mem_read(STACK,64))==stack
   assert callbacks==(['entry']if site in [4,5]else['entry','return'])
   checks+=1
report={'status':'PASSED','compiled_nro':True,'calls':checks,'gateways':9,'preserved':['r0-r12','lr','sp','d0-d31','FPSCR','NZCVQ','native stack arguments'],'GE':'all 16 preserved and deliberately changed callback patterns checked; not restored','runtime_test':'NOT RUN'}
(BUILD_ROOT/'gateways.json').write_text(json.dumps(report,indent=2)+'\n');print(report)
