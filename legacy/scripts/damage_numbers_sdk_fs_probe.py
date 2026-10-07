from paths import MOD, SCRIPTS, FORGE, DEPS, PYTHON, CONTAINER_STORAGE, input_path
#!/usr/bin/env python3
"""Execute the pinned SDK's thread-local filesystem abort guard; no host SD I/O."""
import json
import struct
from unicorn import Uc,UC_ARCH_ARM,UC_MODE_ARM,UC_HOOK_CODE
from unicorn import arm_const as arm
from damage_numbers_sdk_tls_probe import sdk_image,ROOT

def main():
    image=sdk_image()
    assert image[0x4cffc:0x4d004].hex()=='0200a0e31eff2fe1'
    assert image[0x4d004:0x4d014].hex()=='30482de908b08de20040a0e130009fe5'
    sdk,data,stop=0x60000000,0x33000000,0x33003000
    u=Uc(UC_ARCH_ARM,UC_MODE_ARM);u.mem_map(sdk,(len(image)+0x4000+4095)&~4095)
    u.mem_write(sdk,image);u.mem_map(data,0x4000)
    # Dynamic context addresses are explicit initialized-SDK fixtures.
    # Their GOT symbols and relative offsets were checked in the original NSO.
    default,disabled,custom=sdk+0x862c90,sdk+0x862c94,data+0x1000
    for address,value in [(sdk+0x85bc60,default),(sdk+0x85bc64,disabled),
                          (sdk+0x862c8c,13),(default,data+0x1800),
                          (disabled,sdk+0x4cffc),(custom,data+0x1800)]:
        u.mem_write(address,struct.pack('<I',value))
    u.mem_write(data+0x1800,bytes.fromhex('0100a0e31eff2fe1')) # Previous policy fixture: Abort.
    contexts={0:custom,1:default};thread=0;checks=0
    def ret(value):
        u.reg_write(arm.UC_ARM_REG_R0,value);u.reg_write(arm.UC_ARM_REG_PC,u.reg_read(arm.UC_ARM_REG_LR))
    def hook(uc,address,size,_):
        if address==stop:uc.emu_stop();return
        if address==sdk+0x43b7b0:
            assert uc.reg_read(arm.UC_ARM_REG_R0)==13;ret(contexts[thread]);return
        if address==sdk+0x43b7bc:
            assert uc.reg_read(arm.UC_ARM_REG_R0)==13
            contexts[thread]=uc.reg_read(arm.UC_ARM_REG_R1);ret(data+0x1800+4*13);return
        if address==sdk+0x43ba74:uc.reg_write(arm.UC_ARM_REG_PC,sdk+0x4d054);return
        if not (sdk+0x4cffc<=address<sdk+0x4d094 or sdk+0x51b64<=address<sdk+0x51c30 or data+0x1800<=address<data+0x1808):
            raise ValueError(f'Unexpected native FS guard dependency {address-sdk:#x}')
    u.hook_add(UC_HOOK_CODE,hook)
    def call(offset,r0):
        u.reg_write(arm.UC_ARM_REG_SP,data+0x3ff0);u.reg_write(arm.UC_ARM_REG_LR,stop)
        u.reg_write(arm.UC_ARM_REG_R0,r0);u.emu_start(sdk+offset,stop,count=200)
        assert u.reg_read(arm.UC_ARM_REG_PC)==stop
        return u.reg_read(arm.UC_ARM_REG_R0)
    for initial in (custom,default,0):
        contexts[0]=initial;contexts[1]=custom
        call(0x4d004,data)
        previous=struct.unpack('<I',u.mem_read(data,4))[0]
        assert previous==(initial or default) and contexts[0]==disabled and contexts[1]==custom
        checks+=3
        for error in (0,1,0x202,0x5802,0x195802,0xffffffff):
            assert call(0x51b64,error)==0;checks+=1
        thread=1
        assert call(0x51b64,0x195802)==1;checks+=1
        thread=0;call(0x4d080,previous)
        assert contexts[0]==previous and call(0x51b64,0x195802)==1;checks+=2
    path=ROOT/'work/damage-numbers/open-assets/sdk-fs-guard-test.json'
    path.write_text(json.dumps({'checks':checks,'native_offsets':{
        'scoped_constructor':'0x4d004','context_setter':'0x4d080','abort_decision':'0x51b64',
        'all_return_handler':'0x4cffc'},'fixtures':'SDK FS slot 13, initialized context GOT/data, previous Abort callback, thread-local getter/setter; no mount or storage trial'},indent=2)+'\n')
    print(f'Passed {checks} original SDK filesystem guard checks; no emulator or host SD writes')

if __name__=='__main__':main()
