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
    check(commands[0].pixels==30&&commands[0].opacity==1&&commands[0].critical==Critical::Positive,"pop and overscore command");
    check(layout(m,view(),1.2f,commands)==1&&commands[0].pixels==24,"pop settles");
    const float old_y=commands[0].position.y;
    check(layout(m,view(),1.8f,commands)==1&&commands[0].position.y<old_y,"upward drift");
    check(std::fabs(commands[0].opacity-0.4f)<0.00001f,"last quarter-second fade");
    check(layout(m,view(),2,commands)==0,"no expired draw");
    e=hit(2);e.critical=Critical::Negative;m.capture(e);m.update(1,true);
    check(layout(m,view(),1,commands)==2,"same-position multi-hit preserved");
    check(commands[1].critical==Critical::Negative&&commands[1].star.y>commands[1].ink.bottom,"negative star below visible glyphs");
    check(commands[0].star.y<commands[0].ink.top,"positive star above visible glyphs");
    check(std::fabs(commands[1].offset.x)<=38.4f&&std::fabs(commands[1].offset.y)<=24,"bounded nearby placement");
    v=view();v.height=1440;v.width=2560;
    check(layout(m,v,1.2f,commands)==2&&commands[0].pixels==48,"resolution-scaled text");
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
    Settings decoded{};
    check(decode_settings(saved.bytes,saved.size,decoded) && !decoded.enabled && decoded.style==VisualStyle::Classic && decoded.scale==1.75f && decoded.lifetime==1.5f,"INI settings round trip");
    const char* invalid[]={"", "[DamageNumbers]\nversion=1\n", "garbage", "[Other]\nversion=1\n"};
    for (const char* text:invalid) check(!decode_settings(text,unsigned(std::strlen(text)),decoded),"incomplete settings rejected");
    const char* values[]={"NaN","inf","-1","4.0","1e0","1.1234567",".","1..2"};
    for (const char* value:values) {
        char text[256]; const int size=std::snprintf(text,sizeof(text),
            "[DamageNumbers]\nversion=2\nenabled=1\nstyle=wilds\nscale=%s\nlifetime=0.9\n",value);
        check(!decode_settings(text,unsigned(size),decoded),"invalid INI numeric value rejected");
    }
    auto corrupt=saved; corrupt.bytes[corrupt.size++]='\0';
    check(!decode_settings(corrupt.bytes,corrupt.size,decoded),"embedded INI NUL rejected");
    char duplicate[256]; std::memcpy(duplicate,saved.bytes,saved.size);
    std::memcpy(duplicate+saved.size,"enabled=1\n",10);
    check(!decode_settings(duplicate,saved.size+10,decoded),"duplicate config key rejected");
    const char* edited="; user settings\r\n[DamageNumbers]\r\nversion = 2\r\nenabled = 0\r\nstyle=wilds\r\nscale=2.5\r\nlifetime=1\r\n";
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
    auto old=saved;for(unsigned k=0;k<old.size;++k)if(old.bytes[k]=='2'){old.bytes[k]='1';break;}
    Settings fresh{};check(!decode_settings(old.bytes,old.size,fresh)&&fresh.enabled&&fresh.style==VisualStyle::Wilds&&fresh.scale==1,"v1 cannot preserve old preferences");
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
    check(std::fabs(commands[0].advance-25.2f)<.001f,"actual advances used instead of digit count estimate");
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
int main() { policies();lifecycle();drawing();compact_spacing();aligned_markers();preview();settings_and_sources();std::printf("Passed %u core/presentation checks\n",cases); }
