#include "camera.h"
#include "outcomes.h"
#include "preview.h"
#include "settings.h"
#include "bounce.h"
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
        check(visible(e,s)==((source==0 && owner<2)||source==1||source==2),"default source and ownership matrix");
        Settings enabled=s;enabled.enabled=false;
        check(visible(e,enabled)==false,"master disable overrides every source");
    }
    e=hit();
    for(int n:{-123,0,1,123,2147483647}) {
        e.total_damage=n;check(visible(e,s)==(n>0),"healing, zero and positive amounts");
    }
    e=hit();check(tint(e,s)==Tint::White,"HZV 44");
    e.physical_hzv=45;check(tint(e,s)==Tint::Weak,"HZV 45");
    e.total_damage=999;check(tint(e,s)==Tint::Weak,"elemental total does not select color");
    e.bounced=true;check(tint(e,s)==Tint::Gray,"actual bounce wins color");
    e.bounced=false;
    e.physical_hzv=-1;check(tint(e,s)==Tint::White,"unknown physical HZV");
    for(Critical c:{Critical::None,Critical::Positive,Critical::Negative}) {
        e.critical=c;check(marker(e)==c,"direct critical outcomes");
        e.source=Source::Tick;check(marker(e)==Critical::None,"ticks have no critical marker");
        check(!visible(e,s),"unsupported recurring ticks hidden");e.source=Source::Attack;
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
    s.enabled=true;check(m.configure(s),"enable objects");
    a=hit(50);a.key.epoch=8;a.source=Source::Object;a.owner=Owner::RemotePlayer;
    check(m.capture(a),"recognized remote object always allowed");
    s.enabled=false;m.configure(s);s.enabled=true;m.configure(s);m.update(1,true);
    check(m.size()==0,"master disable discards pending events before re-enable");
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
    check(std::fabs(commands[0].pixels-24*peak_scale(VisualStyle::Wilds))<.001f&&commands[0].opacity==1&&commands[0].critical==Critical::Positive,"pop and overscore command");
    check(layout(m,view(),1.2f,commands)==1&&commands[0].pixels==24,"pop settles");
    const float old_y=commands[0].position.y;
    check(layout(m,view(),1.8f,commands)==1&&commands[0].position.y<old_y,"upward drift");
    check(std::fabs(commands[0].opacity-number_motion(.8f,.9f,VisualStyle::Wilds).opacity)<0.00001f,"last quarter-second fade");
    check(layout(m,view(),2,commands)==0,"no expired draw");
    e=hit(2);e.critical=Critical::Negative;m.capture(e);m.update(1,true);
    check(layout(m,view(),1,commands)==2,"same-position multi-hit preserved");
    check(commands[1].critical==Critical::Negative&&commands[1].star.y>commands[1].ink.bottom,"negative star below visible glyphs");
    check(commands[0].star.y<commands[0].ink.top,"positive star above visible glyphs");
    check(std::fabs(commands[1].offset.x)<=38.4f&&std::fabs(commands[1].offset.y)<=24,"bounded nearby placement");
    v=view();v.height=1440;v.width=2560;
    check(layout(m,v,1.2f,commands)==2&&commands[0].pixels==48,"resolution-scaled text");
    v=view();v.height=1080;v.width=1920;
    check(layout(m,v,1.2f,commands)==2&&commands[0].pixels==36,"default 1x is 36 pixels at 1080p");
    Settings resized=m.config();resized.scale=1.2f;m.configure(resized);
    check(layout(m,view(),1.2f,commands)==2&&std::fabs(commands[0].pixels-28.8f)<.001f,"1.2x multiplies the 24-pixel base");
    const float native_view[16]={1,0,0,0,0,1,0,0,0,0,1,0,0,0,-10,1};
    const float native_projection[16]={1,0,0,0,0,1,0,0,0,0,-2,-1,0,0,-2,0};
    check(from_native_camera(native_view,native_projection,0,0,1280,720,v),"native matrix convention converted");
    check(project({0,0,8.5f},v,p)&&p.x==640&&p.y==360,"native matrix translation and depth");
    check(!project({0,0,12},v,p),"native matrix rejects behind-eye position");
    check(!from_native_camera(native_view,native_projection,1280,0,0,720,v),"invalid native rectangle");
}
static void preview() {
    const float identity[16]={1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1};
    int32_t rect[4]={0,0,1280,720};View v{};Point p{};
    check(preview_camera(identity,identity,rect,2560,1440,v) &&
          project({0,0,.5f},v,p) && p.x==1280 && p.y==720,"native render scale maps to Forge swapchain coordinates");
    check(!preview_camera(identity,identity,rect,1024,768,v),"unknown aspect mapping is hidden");
    rect[0]=10;check(!preview_camera(identity,identity,rect,1280,720,v),"offset viewport requires a verified framebuffer binding");
    rect[0]=0;rect[2]=0;check(!preview_camera(identity,identity,rect,1280,720,v),"empty native framebuffer rectangle rejected");
    rect[2]=1280;check(!preview_camera(identity,identity,rect,NAN,720,v),"invalid Forge display hidden");
    float seconds=0;const uint64_t origin=0x1ffffffffull;
    check(preview_seconds(origin+19200000,origin,19200000,seconds) && seconds==1,
          "64-bit timestamp subtraction preserves elapsed time across low-word wrap");
    check(!preview_seconds(origin-1,origin,19200000,seconds),"timestamp before origin is rejected");
    check(!preview_seconds(origin,origin,0,seconds),"zero clock frequency is rejected");
    check(preview_fresh(origin,origin+1920000,19200000,.2f),"fresh camera accepted");
    check(!preview_fresh(origin,origin+3840000,19200000,.2f),"camera stale at 200ms is hidden");
    check(!preview_fresh(origin,origin-1,19200000,.2f),"future camera tick is hidden");
    check(!preview_fresh(0,origin,19200000,.2f),"absent camera is hidden");
    check(preview_source(0x000100,Owner::LocalPlayer)&&preview_source(0x030403,Owner::LocalPalico),"direct hunter and palico sources accepted");
    check(!preview_source(0x000101,Owner::LocalPlayer),"unclassified hunter secondary ownership stays diagnostic");
    check(preview_source(0x030402,Owner::LocalPalico)&&preview_source(0x030802,Owner::LocalPalico),"observed local-palico child projectile families accepted");
    check(preview_source(0x030402,Owner::LocalPlayer)&&preview_source(0x030802,Owner::LocalPlayer),"local Prowler child attacks accepted");
    for (Owner owner:{Owner::RemotePlayer,Owner::RemotePalico,Owner::Unknown})
        check(!preview_source(0x030402,owner)&&!preview_source(0x030802,owner),"palico projectile exception requires resolved local-palico ownership");
    check(!preview_source(0x030902,Owner::LocalPalico),"unclassified palico objects stay diagnostic");
    check(imgui_color(0xffbf3f,1)==0xff3fbfff,"orange-yellow converts to ImGui ABGR");
    check(imgui_color(0xffffff,.5f)==0x80ffffff,"fade converts to rounded alpha");
    check(imgui_color(0x9a9a9a,-1)==0x009a9a9a&&imgui_color(0x9a9a9a,2)==0xff9a9a9a,"opacity bounds preserve RGB");
    check(imgui_color(0xffffff,NAN)==0,"invalid alpha is transparent");
}
static void settings_and_sources() {
    Settings s{}; s.enabled=false;s.style=VisualStyle::Classic;
    s.scale=1.75f; s.lifetime=1.5f;
    const auto saved=encode_settings(s);
    check(!std::strstr(saved.bytes,"version="),"new INI omits the version marker");
    Settings decoded{};
    check(decode_settings(saved.bytes,saved.size,decoded) && !decoded.enabled && decoded.style==VisualStyle::Classic && decoded.scale==1.75f && decoded.lifetime==1.5f,"INI settings round trip");
    const char* invalid[]={"", "[DamageNumbers]\nversion=1\n", "garbage", "[Other]\nversion=1\n"};
    for (const char* text:invalid) check(!decode_settings(text,unsigned(std::strlen(text)),decoded),"incomplete settings rejected");
    const char* values[]={"NaN","inf","-1","4.0","1e0","1.1234567",".","1..2"};
    for (const char* value:values) {
        char text[256]; const int size=std::snprintf(text,sizeof(text),
            "[DamageNumbers]\nenabled=1\nstyle=wilds\nscale=%s\nlifetime=0.9\n",value);
        check(!decode_settings(text,unsigned(size),decoded),"invalid INI numeric value rejected");
    }
    auto corrupt=saved; corrupt.bytes[corrupt.size++]='\0';
    check(!decode_settings(corrupt.bytes,corrupt.size,decoded),"embedded INI NUL rejected");
    char duplicate[256]; std::memcpy(duplicate,saved.bytes,saved.size);
    std::memcpy(duplicate+saved.size,"enabled=1\n",10);
    check(!decode_settings(duplicate,saved.size+10,decoded),"duplicate config key rejected");
    const char* edited="; user settings\r\n[DamageNumbers]\r\nenabled = 0\r\nstyle=wilds\r\nscale=2.5\r\nlifetime=1\r\n";
    check(decode_settings(edited,unsigned(std::strlen(edited)),decoded) && decoded.scale==2.5f && decoded.lifetime==1,
          "INI whitespace and Windows line endings supported");
    for (float scale:{0.5f,1.0f,1.751234f,3.0f}) for (float lifetime:{0.4f,0.9f,1.231234f,2.0f}) {
        s.scale=scale;s.lifetime=lifetime;const auto record=encode_settings(s);
        check(decode_settings(record.bytes,record.size,decoded) && std::fabs(decoded.scale-scale)<0.000001f &&
              std::fabs(decoded.lifetime-lifetime)<0.000001f,"INI decimals preserve UI values to six places at range boundaries");
    }
    for(const char* key:{"monsters=1\n","objects=0\n","bounce_gray=1\n","style=future\n","version=1\n"}) {
        auto bad=saved;const unsigned n=std::strlen(key);std::memcpy(bad.bytes+bad.size,key,n);bad.size+=n;
        check(!decode_settings(bad.bytes,bad.size,decoded),"legacy, duplicate and invalid fields rejected");
    }
    for (const char* style:{"wilds","classic","rise"}) {
        char legacy[256]; const int size=std::snprintf(legacy,sizeof(legacy),
            "[DamageNumbers]\nversion = 2\nenabled=0\nstyle=%s\nscale=1.75\nlifetime=1.5\n",style);
        check(decode_settings(legacy,unsigned(size),decoded) && !decoded.enabled && decoded.scale==1.75f && decoded.lifetime==1.5f,
              "existing version-2 config keeps all settings");
        const auto rewritten=encode_settings(decoded);
        check(!std::strstr(rewritten.bytes,"version=") && decode_settings(rewritten.bytes,rewritten.size,decoded),
              "existing config saves without a version marker");
        std::strcat(legacy,"version=2\n");
        check(!decode_settings(legacy,unsigned(std::strlen(legacy)),decoded),"duplicate obsolete version marker rejected");
    }
    for (const char* key:{"enabled=", "style=", "scale=", "lifetime="}) {
        char missing[256]; std::memcpy(missing,saved.bytes,saved.size+1);
        char* begin=std::strstr(missing,key); char* end=std::strchr(begin,'\n')+1;
        std::memmove(begin,end,std::strlen(end)+1);
        check(!decode_settings(missing,unsigned(std::strlen(missing)),decoded),"each of four settings remains required");
    }
    Source source=Source::Attack;
    check(source_outcome(0x000102,Owner::LocalPlayer,source) && source==Source::Attack,"hunter projectile remains owned attack");
    check(source_outcome(0x030802,Owner::LocalPalico,source) && source==Source::Attack,"palico support barrel remains owned attack");
    check(source_outcome(0x010202,Owner::Unknown,source) && source==Source::Monster,"secondary monster attack classified");
    check(source_outcome(0x000201,Owner::Unknown,source) && source==Source::Monster,"direct monster attack classified");
    check(source_outcome(0x000802,Owner::RemotePlayer,source) && source==Source::Object,"placed object accepts remote owner as a recognized object");
    check(source_outcome(0x000102,Owner::RemotePlayer,source) && source==Source::Attack,"remote ranged weapon cannot be relabeled an object");
    check(!source_outcome(0x030902,Owner::LocalPalico,source),"unknown child source stays hidden");
    BounceAttack attacks[2]={{100,0,1,0,0},{200,0,0,0xff,0}};
    check(bounce_match(0,attacks,100,0,0,0,0x000100,Owner::LocalPlayer),"unique native bounce feedback matches latest owned melee hit");
    check(!bounce_match(2,attacks,100,0,0,0,0x000100,Owner::LocalPlayer),"native prevention feedback suppresses gray");
    attacks[1].active=1;
    check(!bounce_match(0,attacks,100,0,0,0,0x000100,Owner::LocalPlayer),"two active attacks prevent bounce attribution");
    attacks[1].active=0;
    check(!bounce_match(0,attacks,101,0,0,0,0x000100,Owner::LocalPlayer),"reused or different attack identity rejected");
    check(!bounce_match(0,attacks,100,1,0,0,0x000100,Owner::LocalPlayer),"changed flags rejected");
    check(!bounce_match(0,attacks,100,0,0,0,0x030402,Owner::LocalPalico),"palico projectile cannot inherit hunter bounce");
    Model model; model.reset(7); const Event first=hit(1), second=hit(2);
    model.capture(first); model.capture(second);
    check(model.bounce(second.key),"bounce resolution reaches queued hit");
    model.update(1.05f,true);
    check(!model.at(0).event.bounced && model.at(1).event.bounced,"bounce affects only its exact impact key");
    check(model.bounce(first.key) && model.at(0).event.bounced,"bounce resolution reaches visible hit");
    model.reset(8);
    check(!model.bounce(first.key),"context transition rejects stale bounce resolution");
}
static bool varied_measure(const char* text,float pixels,VisualStyle,TextMetrics& m) {
    float advance=0;for(const char* p=text;*p;++p)advance+=pixels*(*p=='1'?.28f:.62f);
    m={advance,{pixels*.02f,pixels*.14f,advance-pixels*.02f,pixels*.86f}};return true;
}
static void compact_spacing() {
    Model m;m.reset(7);PlacementState state{};DrawNumber commands[128]{};
    Event e=hit();e.total_damage=111;e.critical=Critical::Positive;m.capture(e);m.update(1,true);
    check(layout(m,view(),1,commands,state,varied_measure)==1,"measured narrow digits draw");
    const Point first=commands[0].offset;
    check(first.x==0&&first.y==0,"isolated hit retains its original anchor");
    check(std::fabs(commands[0].advance-(24*peak_scale(VisualStyle::Wilds)*.28f*3))<.001f,"actual advances used instead of digit count estimate");
    check(commands[0].color==0xf4f1ec&&star_rgb(commands[0].critical)==0xf1689b,"nonweak positive crit keeps white digits and red star");
    for(unsigned i=2;i<=128;++i) { e=hit(i);e.total_damage=i%2?888:1;e.critical=i%3?Critical::Negative:Critical::Positive;m.capture(e); }
    m.update(1,true);check(layout(m,view(),1,commands,state,varied_measure)==128,"dense bursts retain all 128 separate numbers");
    Point offsets[128];
    for(unsigned i=0;i<128;++i) {
        offsets[i]=commands[i].offset;
        check(std::fabs(offsets[i].x)<=38.4001f&&std::fabs(offsets[i].y)<=24.0001f,"dense placement remains inside horizontal and vertical bounds");
        check(inside(commands[i].solid,view()),"solid label inside viewport");
    }
    check(layout(m,view(),1.07f,commands,state,varied_measure)==128,"burst animates without losses");
    for(unsigned i=0;i<128;++i)check(offsets[i].x==commands[i].offset.x&&offsets[i].y==commands[i].offset.y,"existing hits never reshuffle");
    Settings s=m.config();s.style=VisualStyle::Classic;m.configure(s);
    layout(m,view(),1.07f,commands,state,varied_measure);
    check(state.style==VisualStyle::Classic&&commands[0].color==0xffffff,"Classic restores old palette and resets placement generation");
    s.scale=2;m.configure(s);View v=view();v.width=2560;v.height=1440;
    layout(m,v,1.07f,commands,state,varied_measure);
    check(state.scale==2&&state.width==2560,"size and viewport changes refresh reservation dimensions");
    m.reset(8);e=hit();e.key.epoch=8;m.capture(e);m.update(1,true);
    check(layout(m,view(),1,commands,state,varied_measure)==1&&commands[0].offset.x==0&&commands[0].offset.y==0,"context transition clears cached placements");
    m.reset(7);e=hit();e.position.x=-.98f;m.capture(e);m.update(1,true);
    check(layout(m,view(),1,commands,state,varied_measure)==1&&inside(commands[0].reserve,view()),"viewport edge chooses a nearby inward position");
    check(layout(m,view(),2,commands,state,varied_measure)==0,"expired labels are not rendered");
}
static void aligned_markers() {
    const Rect ink={0,4,40,24};
    const Rect positive=marker_bounds(ink,30,1,Critical::Positive,VisualStyle::Wilds);
    const Rect negative=marker_bounds(ink,30,1,Critical::Negative,VisualStyle::Wilds);
    check(std::fabs(positive.top-(4-2-30*wilds_star_height))<.001f&&std::fabs(negative.bottom-(24+2+30*wilds_star_height))<.001f,"original sprite solid bounds reserve measured alpha-core height plus glyph gap");
    Model m;m.reset(7);Event e=hit();e.position={.2517f,.013f,.5f};e.critical=Critical::Positive;m.capture(e);m.update(1,true);
    DrawNumber out[128]{};PlacementState state{};
    layout(m,view(),1,out,state,varied_measure);
    check(out[0].position.x==int(out[0].position.x)&&out[0].position.y==int(out[0].position.y),"Wilds text origin is pixel aligned");
    check(std::fabs(out[0].star.x-(out[0].ink.left+out[0].ink.right)*.5f)<.001f,"star follows actual visible glyph center");
    Settings s=m.config();s.style=VisualStyle::Classic;m.configure(s);layout(m,view(),1,out,state,varied_measure);
    check(out[0].position.x!=int(out[0].position.x),"Classic retains original text origin behavior");
}
static void stamp_hold_exit() {
    Model m;m.reset(7);Event e=hit();e.critical=Critical::Positive;m.capture(e);m.update(1,true);
    Settings s=m.config();s.lifetime=1.2f;m.configure(s);
    PlacementState cache{};DrawNumber out[128]{};
    check(layout(m,view(),1,out,cache,varied_measure)==1&&std::fabs(out[0].pixels-28.8f)<.001f,
          "open Wilds starts with 1.2x stamp");
    const float start_y=out[0].position.y;const Point offset=out[0].offset;
    check(layout(m,view(),1.1f,out,cache,varied_measure)==1&&std::fabs(out[0].pixels-25.2f)<.001f,
          "stamp eases down halfway through the 0.2-second settlement");
    check(layout(m,view(),1.2f,out,cache,varied_measure)==1&&out[0].pixels==24,
          "stamp reaches normal size at 0.2 seconds");
    for(float age:{.25f,.5f,.75f,.95f,1.0f}) {
        check(layout(m,view(),1+age,out,cache,varied_measure)==1&&out[0].position.y==start_y&&out[0].opacity>.999f,
              "one-second stationary fully opaque hold with total duration 1.2");
        check(out[0].offset.x==offset.x&&out[0].offset.y==offset.y,"hold keeps cached spacing fixed");
    }
    check(layout(m,view(),2.1f,out,cache,varied_measure)==1&&std::fabs(out[0].position.y-(start_y-15))<=1&&std::fabs(out[0].opacity-.5f)<.001f,
          "final 0.2 seconds rises fast while fading, halfway at 15 pixels and half alpha");
    check(out[0].pixels==24&&out[0].star.y<out[0].ink.top,"exit keeps settled size and positive star attached");
    check(layout(m,view(),2.2f,out,cache,varied_measure)==0,"label disappears at total duration");
    s.lifetime=.9f;m.configure(s);layout(m,view(),1.65f,out,cache,varied_measure);
    check(out[0].position.y==start_y&&out[0].opacity==1,"default duration holds beyond half a second");
    s.lifetime=.4f;m.configure(s);layout(m,view(),1.3f,out,cache,varied_measure);
    check(std::fabs(out[0].opacity-.5f)<.001f&&out[0].pixels==24,"minimum duration fits stamp and exit without negative hold");
    s.lifetime=1.2f;m.configure(s);View large=view();large.width*=2;large.height*=2;
    layout(m,large,1.5f,out,cache,varied_measure);const float large_y=out[0].position.y;
    layout(m,large,2.1f,out,cache,varied_measure);
    check(std::fabs(out[0].position.y-(large_y-30))<=1,"exit distance scales with viewport resolution");
    s.style=VisualStyle::Classic;s.lifetime=.9f;m.configure(s);
    layout(m,view(),1,out,cache,varied_measure);const float classic_y=out[0].position.y;
    check(out[0].pixels==30,"Classic keeps 1.25x pop");
    layout(m,view(),1.1f,out,cache,varied_measure);check(out[0].pixels==24&&out[0].position.y<classic_y,
          "Classic keeps 0.1-second settlement and continuous upward drift");
    layout(m,view(),1.8f,out,cache,varied_measure);check(std::fabs(out[0].opacity-.4f)<.001f,
          "Classic keeps its quarter-second fade");
}
static void rise_candidate() {
    check(unsigned(VisualStyle::Wilds)==0&&unsigned(VisualStyle::Classic)==1&&unsigned(VisualStyle::Rise)==2,"style IDs append Rise without changing old IDs");
    for(VisualStyle style:{VisualStyle::Wilds,VisualStyle::Classic,VisualStyle::Rise}) {
        Settings s;s.style=style;Settings decoded{};auto text=encode_settings(s);
        check(decode_settings(text.bytes,text.size,decoded)&&decoded.style==style,"versionless styles round trip");
        for(Owner owner:{Owner::LocalPlayer,Owner::LocalPalico}) for(int hz:{-1,44,45,100})
            for(bool bounce:{false,true}) for(Critical crit:{Critical::None,Critical::Positive,Critical::Negative}) {
                Event e=hit();e.owner=owner;e.physical_hzv=hz;e.bounced=bounce;e.critical=crit;
                Tint expected=bounce?Tint::Gray:hz>=45?Tint::Weak:Tint::White;
                check(tint(e,s)==expected&&marker(e)==crit,"all styles share bounce priority and HZV independently of ownership/affinity");
                if(style==VisualStyle::Rise)check(rgb(tint(e,s),style)==(bounce?0x808080u:hz>=45?0xf0b41eu:0xffffffu),"Rise agreed palette, no companion gray");
            }
    }
    for(float lifetime:{.4f,.9f,2.0f}) {
        const float entrance=low(.3f,lifetime*.3f),exit_time=low(10.0f/30,lifetime*.35f),start=lifetime-exit_time;
        auto a=number_motion(0,lifetime,VisualStyle::Rise);
        auto b=number_motion(entrance*2/9,lifetime,VisualStyle::Rise);
        auto c=number_motion(entrance*7/9,lifetime,VisualStyle::Rise);
        auto d=number_motion(entrance,lifetime,VisualStyle::Rise);
        check(std::fabs(a.scale-.64f)<.001f&&std::fabs(b.scale-1.2f)<.001f&&std::fabs(c.scale-.88f)<.001f&&std::fabs(d.scale-1)<.001f,"Rise grows/shrinks/settles");
        check(number_motion(start,lifetime,VisualStyle::Rise).rise==0,"Rise holds to exit");
        check(std::fabs(number_motion(start+exit_time*.1f,lifetime,VisualStyle::Rise).rise+4)<.001f,"brief downward kick");
        check(number_motion(start+exit_time*.5f,lifetime,VisualStyle::Rise).rise>0,"exit rises after kick");
        auto end=number_motion(lifetime,lifetime,VisualStyle::Rise);
        check(std::fabs(end.rise-50)<.001f&&end.opacity<.001f,"authored exit reaches minus fifty and zero alpha");
        for(unsigned i=0;i<unsigned(lifetime*30);++i) {
            float t=float(i)/30;auto f30=number_motion(t,lifetime,VisualStyle::Rise),f60=number_motion(float(i*2)/60,lifetime,VisualStyle::Rise);
            check(f30.scale==f60.scale&&f30.rise==f60.rise&&f30.opacity==f60.opacity,"30/60fps elapsed-time equivalence");
        }
    }
    Settings s;s.style=VisualStyle::Rise;Model m;m.reset(7);m.configure(s);
    for(unsigned i=1;i<=4;++i){Event e=hit(i);e.critical=i%2?Critical::Positive:Critical::Negative;m.capture(e);}
    m.update(1,true);PlacementState cache;DrawNumber n[128]{};View v=view();
    check(layout(m,v,1.05f,n,cache)==4,"Rise retains multiple same-frame finalized hits");
    bool displaced=false;Point offsets[4];float slopes[4];
    for(unsigned i=0;i<4;++i) {
        offsets[i]=n[i].offset;slopes[i]=n[i].slash.slope;displaced|=offsets[i].x!=0||offsets[i].y!=0;
        check(n[i].slash.origin.x==640&&n[i].slash.origin.y==360,"slash stays at raw hit despite digit collisions and anchor correction");
        check(slopes[i]>=-.25f&&slopes[i]<=.25f,"stable hit tilt bounded to approximately fourteen degrees");
        check(n[i].slash.opacity>0&&std::fabs(n[i].slash.extent-540)<.001f,"selected three-quarter-height peak slash uses gameplay viewport");
    }
    check(displaced,"crowded fixture actually displaces digits");
    v.world_to_clip[3]=.1f;layout(m,v,1.1f,n,cache);
    for(unsigned i=0;i<4;++i)check(std::fabs(n[i].slash.origin.x-704)<.001f&&n[i].slash.origin.y==360&&n[i].slash.slope==slopes[i]&&n[i].offset.x==offsets[i].x&&n[i].offset.y==offsets[i].y,"camera reprojection moves registered hit while cached digit placement and tilt persist");
    layout(m,v,1.3f,n,cache);for(unsigned i=0;i<4;++i)check(n[i].opacity==1&&n[i].slash.opacity==0,"slash expires independently while digits held");
    Event e=hit();Model weak;weak.reset(7);weak.configure(s);e.physical_hzv=45;weak.capture(e);weak.update(1,true);
    layout(weak,view(),1.3f,n,cache);check(n[0].pixels==28,"orange settles at twenty eight versus normal twenty");
    Slash beam=rise_slash({640,360},hit().key,.05f,view(),.75f);Point quad[4];slash_quad(beam,quad);
    check(quad[2].y-quad[0].y==540&&std::fabs(quad[0].x+beam.width*.5f-640-beam.slope*540*.5f)<.001f,"three-quarter height tilted quad centers equal upward/downward reach");
    check(std::fabs((quad[0].y+quad[2].y)*.5f-360)<.001f,"crit beam is centered at the registered hit, without a ground cutoff");
    v=view();v.world_to_clip[15]=-1;check(layout(m,v,1.3f,n,cache)==0,"Rise rejects behind-camera hit");
    for(unsigned i=1;i<2048;++i)check(std::fabs(slash_slope({7,i,20,0}))<=.25f,"all deterministic tilt samples in bounds");
}

int main() {rise_candidate(); policies();lifecycle();drawing();compact_spacing();aligned_markers();stamp_hold_exit();preview();settings_and_sources();std::printf("Passed %u core/presentation checks\n",cases); }
