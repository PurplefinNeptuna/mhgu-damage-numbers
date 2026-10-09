// Independent C++ r25 oracle. Only the test executable contains this reference implementation.
#include "preview.h"
#include <cstdio>
using namespace damage_numbers;
int main(){
 for(int style=0;style<3;++style)for(int crit=0;crit<3;++crit)for(int hz=0;hz<3;++hz)for(int step=0;step<9;++step){
  Model model;model.reset(1);Settings settings;settings.style=VisualStyle(style);model.configure(settings);
  for(unsigned i=0;i<8;++i){Event e{{1,i+1,2,0},{float(int(i%3)-1)*.03f,float(int(i/3)-1)*.02f,.5f},int(i*17+1),int16_t(hz==0?-1:hz==1?44:45),Source::Attack,Owner::LocalPlayer,Critical(crit),false,0};model.capture(e);}model.update(.01f,true);
  View view{{1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1},0,0,1920,1080,DepthRange::ZeroToOne};PlacementState cache;DrawNumber out[128];layout(model,view,.01f,out,cache);float now=step*.1f;model.update(now,true);unsigned count=layout(model,view,now,out,cache);
  std::printf("%d %d %d %d %u\n",style,crit,hz,step,count);
  for(unsigned i=0;i<count;++i){auto &d=out[i];float values[]={d.position.x,d.position.y,d.offset.x,d.offset.y,d.star.x,d.star.y,d.slash.origin.x,d.slash.origin.y,d.slash.slope,d.slash.extent,d.slash.width,d.slash.opacity,d.pixels,d.opacity,d.advance,d.unit,d.classic_stroke,d.ink.left,d.ink.top,d.ink.right,d.ink.bottom,d.solid.left,d.solid.top,d.solid.right,d.solid.bottom,d.reserve.left,d.reserve.top,d.reserve.right,d.reserve.bottom};for(float value:values)std::printf("%.9g ",double(value));std::printf("%u %u %s\n",d.color,d.key.serial,d.text);}
 }
}
