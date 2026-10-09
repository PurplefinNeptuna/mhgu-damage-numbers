#!/usr/bin/env python3
"""Compare the Rust layout/model against the immutable r25 C++ oracle over styles, HZVs, criticals and ages."""
import json,math,subprocess,sys
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'scripts'))
from paths import MOD,BUILD_ROOT
BUILD_ROOT.mkdir(parents=True,exist_ok=True)
cpp=BUILD_ROOT/'parity-oracle'
subprocess.run(['c++','-std=c++17','-O2','-o',str(cpp),str(MOD/'tests/oracle/parity.cpp')],check=True)
expected=subprocess.check_output([cpp],text=True).splitlines()
actual=subprocess.check_output(['cargo','run','--quiet','--config',str(MOD/'.cargo/vendor.toml'),'--manifest-path',str(MOD/'core/Cargo.toml'),'--target','x86_64-unknown-linux-gnu','--example','parity'],cwd=MOD.parent,text=True).splitlines()
assert len(actual)==len(expected),(len(actual),len(expected))
for i,(left,right)in enumerate(zip(expected,actual)):
 a,b=left.split(),right.split();assert len(a)==len(b),(i,a,b)
 for j,(x,y)in enumerate(zip(a,b)):
  if len(a)==5 or j>=29:assert x==y,(i,j,x,y)
  else:assert math.isclose(float(x),float(y),rel_tol=2e-6,abs_tol=2e-4),(i,j,x,y)
report={'status':'PASSED','scenarios':243,'draw_records':len(actual)-243,'float_tolerance':{'relative':2e-6,'absolute':2e-4}}
(BUILD_ROOT/'parity.json').write_text(json.dumps(report,indent=2)+'\n');print(report)

policy=BUILD_ROOT/'policy-oracle'
subprocess.run(['c++','-std=c++17','-O2','-o',str(policy),str(MOD/'tests/oracle/policy.cpp')],check=True)
expected=subprocess.check_output([policy],text=True).splitlines()
actual=subprocess.check_output(['cargo','run','--quiet','--config',str(MOD/'.cargo/vendor.toml'),'--manifest-path',str(MOD/'core/Cargo.toml'),'--target','x86_64-unknown-linux-gnu','--example','policy'],cwd=MOD.parent,text=True).splitlines()
assert len(expected)==len(actual)
for i,(left,right)in enumerate(zip(expected,actual)):
 a,b=left.split(),right.split();assert a[:2]==b[:2]and len(a)==len(b),(i,a,b)
 for x,y in zip(a[2:],b[2:]):assert math.isclose(float(x),float(y),rel_tol=2e-6,abs_tol=2e-4),(i,a,b)
report['policy_and_projection_cases']=len(actual)
(BUILD_ROOT/'parity.json').write_text(json.dumps(report,indent=2)+'\n');print('PASS:',len(actual),'r25 policy/projection cases')
