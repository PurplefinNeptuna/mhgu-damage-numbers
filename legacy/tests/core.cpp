#include "camera.h"
#include "outcomes.h"
#include <cstdio>
#include <cstring>
#include <limits>
#include <cstdlib>
#include <initializer_list>
using namespace damage_numbers;
static unsigned cases=0;
static void check(bool ok,const char* label) {
    ++cases;
    if (!ok) { std::fprintf(stderr,"FAIL: %s\n",label); std::exit(1); }
}
static Event hit(unsigned serial=1) {
    return {{7,serial,20,0},{0,0,0.5f},123,44,Source::Attack,Owner::LocalPlayer,
            Critical::None,false,1.0f};
}
static View view() {
    return {{1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1},0,0,1280,720,DepthRange::ZeroToOne};
}
static void policies() {
    Settings s{}; Event e=hit();
    for (unsigned source=0;source<4;++source) for (unsigned owner=0;owner<5;++owner) {
        e.source=Source(source); e.owner=Owner(owner);
        check(visible(e,s)==(source==0 && owner<2),"default source and ownership matrix");
        Settings enabled=s;enabled.monsters=enabled.objects=enabled.ticks=true;
        check(visible(e,enabled)==(source!=0 || owner<2),"optional all-owner exceptions");
    }
    e=hit();
    for(int n:{-123,0,1,123,2147483647}) {
        e.total_damage=n;check(visible(e,s)==(n>0),"healing, zero and positive amounts");
    }
    e=hit();check(tint(e,s)==Tint::White,"HZV 44");
    e.physical_hzv=45;check(tint(e,s)==Tint::Weak,"HZV 45");
    e.total_damage=999;check(tint(e,s)==Tint::Weak,"elemental total does not select color");
    e.bounced=true;check(tint(e,s)==Tint::Gray,"actual bounce wins color");
    s.bounce_gray=false;check(tint(e,s)==Tint::Weak,"bounce color can be disabled");
    e.physical_hzv=-1;check(tint(e,s)==Tint::White,"unknown physical HZV");
    for(Critical c:{Critical::None,Critical::Positive,Critical::Negative}) {
        e.critical=c;check(marker(e)==c,"direct critical outcomes");
        e.source=Source::Tick;check(marker(e)==Critical::None,"ticks have no critical marker");
        check(tint(e,s)==Tint::Gray,"ticks are gray");e.source=Source::Attack;
    }
    e=hit();e.position.x=std::numeric_limits<float>::quiet_NaN();check(!visible(e,s),"bad hit position");
    e=hit();e.critical=Critical(255);check(!visible(e,s),"unknown critical outcome");
    e=hit();e.owner=Owner(255);check(!visible(e,s),"invalid owner");
    e=hit();e.source=Source(255);check(!visible(e,s),"invalid source");
    s.scale=0;check(!valid(s),"invalid text scale");s.scale=1;s.lifetime=INFINITY;
    check(!valid(s),"invalid lifetime");
    check(!damage_outcome(0x1d,50),"native healing type rejected even with positive intermediate amount");
    check(!damage_outcome(1,0)&&!damage_outcome(1,-50),"nonpositive native contributions rejected");
    check(damage_outcome(1,50),"positive finalized native damage accepted");
    Critical native{};
    check(critical_outcome(0,native)&&native==Critical::None,"native noncritical");
    check(critical_outcome(1,native)&&native==Critical::Negative,"native negative critical");
    check(critical_outcome(2,native)&&native==Critical::Positive,"native positive critical");
    check(!critical_outcome(3,native),"unrecognized native critical rejected");
}
static void lifecycle() {
    Model m;m.reset(7);
    Event a=hit(),b=hit(2),c=hit();c.key.application=1;
    check(m.capture(a),"first impact");check(!m.capture(a),"duplicate callback");
    check(m.capture(b)&&m.capture(c),"same-frame separate impacts and applications");
    m.update(1.0f,true);check(m.size()==3 && m.duplicates==1,"multi-hit preserved");
    m.update(1.8f,true);check(m.size()==3,"numbers retained before expiration");
    m.update(2.0f,true);check(m.size()==0,"expired numbers cleared");
    m.reset(8);check(!m.capture(a),"old quest epoch rejected");a.key.epoch=8;
    check(m.capture(a),"new quest event");m.update(1.0f,false);
    check(m.pending()==0&&m.size()==0,"invalid gameplay clears display and queue");
    check(m.capture(a),"invalid context clears prior event identities");m.update(1,true);
    Settings s=m.config();s.enabled=false;check(m.configure(s)&&m.size()==0,"master disable clears active");
    s.enabled=true;s.objects=true;check(m.configure(s),"enable objects");
    a=hit(50);a.key.epoch=8;a.source=Source::Object;a.owner=Owner::RemotePlayer;
    check(m.capture(a),"remote object allowed when enabled");
    s.objects=false;m.configure(s);s.objects=true;m.configure(s);m.update(1,true);
    check(m.size()==0,"disabling option discards pending events before re-enable");
    s.scale=NAN;check(!m.configure(s)&&m.config().scale==1,"bad config leaves prior settings");
    m.reset(7);
    for(unsigned i=1;i<=300;++i) check(m.capture(hit(i)),"independent burst");
    check(m.pending()==256&&m.queue_drops()==44,"bounded queue drops oldest");
    m.update(1,true);check(m.size()==128&&m.active_drops==128,"bounded visible list");
    check(m.at(0).event.key.serial==173&&m.at(127).event.key.serial==300,"newest burst retained without sums");
    m.reset(7);
    for(unsigned i=1;i<=520;++i) m.capture(hit(i));
    check(!m.capture(hit(520)),"dedup wraps and preserves latest identity");
    check(m.capture(hit(1)),"bounded dedup history evicts oldest identity");
}
static void drawing() {
    View v=view();Point p{};
    check(project({0,0,.5f},v,p)&&p.x==640&&p.y==360,"center projection");
    check(project({-1,1,0},v,p)&&p.x==0&&p.y==0,"viewport orientation and corner");
    check(!project({2,0,.5f},v,p),"outside camera frustum");
    check(!project({0,0,-.5f},v,p),"zero-to-one depth rejects negative");
    v.depth=DepthRange::MinusOneToOne;check(project({0,0,-.5f},v,p),"minus-one depth accepts negative");
    v.world_to_clip[15]=-1;check(!project({0,0,.5f},v,p),"behind camera");
    v=view();v.x=100;v.y=20;v.width=640;v.height=360;
    check(project({0,0,.5f},v,p)&&p.x==420&&p.y==200,"viewport offset and resolution");
    v.width=NAN;check(!project({0,0,.5f},v,p),"invalid viewport");
    char text[12]{};check(digits(2147483647,text)==10&&!std::strcmp(text,"2147483647"),"full integer digits");
    check(digits(-1,text)==0&&text[0]==0,"no negative digits");
    Model m;m.reset(7);Event e=hit();e.critical=Critical::Positive;m.capture(e);m.update(1,true);
    DrawNumber commands[128]{};
    check(layout(m,view(),1,commands)==1,"initial draw");
    check(commands[0].pixels==30&&commands[0].opacity==1&&commands[0].critical==Critical::Positive,"pop and overscore command");
    check(layout(m,view(),1.2f,commands)==1&&commands[0].pixels==24,"pop settles");
    const float old_y=commands[0].position.y;
    check(layout(m,view(),1.8f,commands)==1&&commands[0].position.y<old_y,"upward drift");
    check(std::fabs(commands[0].opacity-0.4f)<0.00001f,"last quarter-second fade");
    check(layout(m,view(),2,commands)==0,"no expired draw");
    e=hit(2);e.critical=Critical::Negative;m.capture(e);m.update(1,true);
    check(layout(m,view(),1,commands)==2,"same-position multi-hit preserved");
    check(commands[1].critical==Critical::Negative&&commands[1].position.y<commands[0].position.y,"separate lane and underscore");
    v=view();v.height=1440;v.width=2560;
    check(layout(m,v,1.2f,commands)==2&&commands[0].pixels==48,"resolution-scaled text");
    const float native_view[16]={1,0,0,0,0,1,0,0,0,0,1,0,0,0,-10,1};
    const float native_projection[16]={1,0,0,0,0,1,0,0,0,0,-2,-1,0,0,-2,0};
    check(from_native_camera(native_view,native_projection,0,0,1280,720,v),"native matrix convention converted");
    check(project({0,0,8.5f},v,p)&&p.x==640&&p.y==360,"native matrix translation and depth");
    check(!project({0,0,12},v,p),"native matrix rejects behind-eye position");
    check(!from_native_camera(native_view,native_projection,1280,0,0,720,v),"invalid native rectangle");
}
int main() { policies();lifecycle();drawing();std::printf("Passed %u core/presentation checks\n",cases); }
