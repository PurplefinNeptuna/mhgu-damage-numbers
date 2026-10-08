#pragma once
#include "model.h"
#include <cstring>

namespace damage_numbers {
static constexpr unsigned settings_text_limit=256;
struct SettingsText { char bytes[settings_text_limit]{}; unsigned size=0; };
// Small bounded INI codec. Decimal values have six places; no locale or heap.
inline SettingsText encode_settings(const Settings& s) {
    SettingsText out{};
    auto text=[&](const char* value) { while (*value && out.size<sizeof(out.bytes)) out.bytes[out.size++]=*value++; };
    auto decimal=[&](float value) {
        const uint32_t fixed=uint32_t(value*1000000.0f+0.5f);
        out.bytes[out.size++]=char('0'+fixed/1000000);
        out.bytes[out.size++]='.';
        for (uint32_t place=100000;place;place/=10) out.bytes[out.size++]=char('0'+(fixed/place)%10);
        out.bytes[out.size++]='\n';
    };
    text("[DamageNumbers]\nenabled="); text(s.enabled?"1\n":"0\n");
    text("style="); text(s.style==VisualStyle::Wilds?"wilds\n":s.style==VisualStyle::Rise?"rise\n":"classic\n");
    text("scale="); decimal(s.scale); text("lifetime="); decimal(s.lifetime);
    return out;
}
inline bool decode_settings(const char* data,unsigned size,Settings& out) {
    if (!size || size>=settings_text_limit) return false;
    Settings next{}; unsigned seen=0; bool section=false;
    auto space=[](char c) { return c==' ' || c=='\t' || c=='\r'; };
    auto equal=[](const char* a,unsigned size,const char* b) { return std::strlen(b)==size && !std::memcmp(a,b,size); };
    for (unsigned start=0;start<size;) {
        unsigned end=start; while (end<size && data[end]!='\n') { if (!data[end]) return false; ++end; }
        unsigned begin=start; start=end+1;
        while (begin<end && space(data[begin])) ++begin;
        while (end>begin && space(data[end-1])) --end;
        if (begin==end || data[begin]==';' || data[begin]=='#') continue;
        if (data[begin]=='[') { section=equal(data+begin,end-begin,"[DamageNumbers]"); continue; }
        if (!section) continue;
        unsigned split=begin; while (split<end && data[split]!='=') ++split;
        if (split==end) return false;
        unsigned key_end=split; while (key_end>begin && space(data[key_end-1])) --key_end;
        unsigned value=split+1; while (value<end && space(data[value])) ++value;
        const char* names[]={"enabled","style","scale","lifetime","version"};
        unsigned key=0; while (key<5 && !equal(data+begin,key_end-begin,names[key])) ++key;
        if (key==5) return false; // No legacy keys or formats.
        if (seen&(1u<<key)) return false;
        seen|=1u<<key;
        if (key==4) {
            // Read existing files, but never write this obsolete format marker.
            if (!equal(data+value,end-value,"2")) return false;
        } else if (key==0) {
            if (end-value!=1 || (data[value]!='0' && data[value]!='1')) return false;
            const bool yes=data[value]=='1';
            next.enabled=yes;
        } else if (key==1) {
            if (equal(data+value,end-value,"wilds")) next.style=VisualStyle::Wilds;
            else if (equal(data+value,end-value,"classic")) next.style=VisualStyle::Classic;
            else if (equal(data+value,end-value,"rise")) next.style=VisualStyle::Rise;
            else return false;
        } else {
            if (value==end) return false;
            unsigned whole=0,digits=0,fraction=0,places=0; bool dot=false;
            for (unsigned i=value;i<end;++i) {
                const char c=data[i];
                if (c=='.' && !dot) { dot=true; continue; }
                if (c<'0' || c>'9') return false;
                ++digits;
                if (dot) { if (++places>6) return false; fraction=fraction*10+unsigned(c-'0'); }
                else { if (whole>3) return false; whole=whole*10+unsigned(c-'0'); }
            }
            if (!digits) return false;
            uint32_t divisor=1; for (unsigned i=0;i<places;++i) divisor*=10;
            const float number=float(whole)+float(fraction)/float(divisor);
            if (key==2) next.scale=number; else next.lifetime=number;
        }
    }
    if ((seen&0x0fu)!=0x0fu || !valid(next)) return false;
    out=next; return true;
}
bool settings_load(Settings& out);
bool settings_save(const Settings& value);
bool settings_pending();
void settings_changed();
const char* settings_status();
}
