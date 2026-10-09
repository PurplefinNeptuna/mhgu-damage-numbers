// Run r25 policy and native-camera fixtures in the same order as the Rust example.
#include "sources.h"
#include "outcomes.h"
#include "camera_bridge.cpp"
#include <cstdio>
#include <initializer_list>
#include <limits>
using namespace damage_numbers;
int main(){
 for(unsigned owner=0;owner<5;++owner)for(unsigned a=0;a<5;++a)for(unsigned b=0;b<10;++b)for(unsigned c=0;c<5;++c){Source s;bool ok=source_outcome(a|(b<<8)|(c<<16),Owner(owner),s);printf("S %u\n",ok?unsigned(s):255);}
 for(unsigned flag=0;flag<256;++flag){Critical c=Critical::None;critical_outcome(flag,c);printf("C %u\n",unsigned(c));}
 for(unsigned type=0;type<256;++type)for(int amount:{-1,0,1,9999})printf("D %u\n",damage_outcome(type,amount));
 for(unsigned camera=0;camera<6;++camera){float v[16]={1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1},p[16];std::memcpy(p,v,sizeof(p));
  if(camera==1)v[12]=.2f;if(camera==2){v[0]=.8f;v[5]=.7f;v[13]=-.15f;}if(camera==3){p[11]=.2f;p[15]=.8f;}if(camera==4)p[0]=std::numeric_limits<float>::quiet_NaN();if(camera==5)p[15]=-1;
  for(float x:{-2.f,-1.f,0.f,.7f,2.f})for(float y:{-1.f,0.f,1.f})for(float z:{-.1f,.5f,2.f}){float point[]={x,y,z},out[2];bool ok=dn_camera_project(v,p,point,out);printf("P %u",ok);if(ok)printf(" %.9g %.9g",out[0],out[1]);printf("\n");}
 }
}
