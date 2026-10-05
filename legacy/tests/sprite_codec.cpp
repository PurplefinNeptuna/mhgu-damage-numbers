#include "sprite_codec.h"
#include <algorithm>
#include <cassert>
#include <chrono>
#include <cstdio>
#include <cstring>
#include <fstream>
#include <iterator>
#include <vector>
using Bytes=std::vector<uint8_t>;
static Bytes load(const char* path) {
    std::ifstream file(path,std::ios::binary);assert(file);
    return {std::istreambuf_iterator<char>(file),std::istreambuf_iterator<char>()};
}
static bool guarded(const Bytes& source,size_t size) {
    Bytes output(size+32,0xab);
    bool ok=damage_numbers::decode_sprite(source.data(),source.size(),output.data()+16,size);
    assert(std::all_of(output.begin(),output.begin()+16,[](uint8_t b){return b==0xab;}));
    assert(std::all_of(output.end()-16,output.end(),[](uint8_t b){return b==0xab;}));
    return ok;
}
static volatile uint8_t sink;
template<typename Decode> static double median_us(Decode decode,const Bytes& source,Bytes& output) {
    std::vector<double> times;
    for(int sample=0;sample<9;++sample) {
        auto start=std::chrono::steady_clock::now();
        for(int i=0;i<100;++i) {decode(source,output);sink=output[i];}
        double us=std::chrono::duration<double,std::micro>(std::chrono::steady_clock::now()-start).count()/100;
        times.push_back(us);
    }
    std::sort(times.begin(),times.end());return times[4];
}
int main(int argc,char** argv) {
    assert(argc==4);const auto packed=load(argv[1]),raw=load(argv[2]),rle=load(argv[3]);
    Bytes output(raw.size());
    assert(damage_numbers::decode_sprite(packed.data(),packed.size(),output.data(),output.size()));
    assert(output==raw);assert(guarded(packed,raw.size()));
    const std::vector<Bytes> invalid={{},{0},{1,42},{128},{128,0},{128,0,0},
        {128,1,0},{255},{255,0},{255,0,0},{255,0,0,1},{0,42,128,0,0},
        {0,42,128,2,0},{0,42,255,255,255,1,0},{0,42,128,1,0}};
    for(const auto& bytes:invalid)assert(!guarded(bytes,3));
    assert(!damage_numbers::decode_sprite(nullptr,1,output.data(),output.size()));
    assert(!damage_numbers::decode_sprite(packed.data(),packed.size(),nullptr,output.size()));
    assert(!guarded(packed,raw.size()-1));assert(!guarded(packed,raw.size()+1));
    auto extra=packed;extra.push_back(0);extra.push_back(0);assert(!guarded(extra,raw.size()));
    for(size_t n=0;n<packed.size();n+=997) {
        Bytes prefix(packed.begin(),packed.begin()+n);assert(!guarded(prefix,raw.size()));
    }
    // Literal runs, short/long matches and overlapping copies.
    Bytes overlap={0,42,128,1,0};Bytes expected(4,42),decoded(4);
    assert(damage_numbers::decode_sprite(overlap.data(),overlap.size(),decoded.data(),decoded.size()));
    assert(decoded==expected);
    Bytes extended={0,42,255,0,0,1,0};decoded.resize(131);
    assert(damage_numbers::decode_sprite(extended.data(),extended.size(),decoded.data(),decoded.size()));
    assert(std::all_of(decoded.begin(),decoded.end(),[](uint8_t b){return b==42;}));
    // Deterministic malformed byte streams under ASan/UBSan and guard checks.
    uint32_t random=0x13579bdf;
    for(unsigned i=0;i<20000;++i) {
        Bytes bytes(i%97+1);
        for(auto& byte:bytes) {random=random*1664525+1013904223;byte=random>>24;}
        guarded(bytes,i%257);
    }
    auto lz=[](const Bytes& source,Bytes& pixels) {
        bool ok=damage_numbers::decode_sprite(source.data(),source.size(),pixels.data(),pixels.size());assert(ok);
    };
    auto old=[](const Bytes& source,Bytes& pixels) {
        size_t written=0;
        for(size_t at=0;at<source.size();at+=5)for(unsigned i=0;i<source[at];++i) {
            const uint32_t word=uint32_t(source[at+1])|(uint32_t(source[at+2])<<8)|
                (uint32_t(source[at+3])<<16)|(uint32_t(source[at+4])<<24);
            std::memcpy(pixels.data()+written,&word,4);written+=4;
        }
    };
    old(rle,output);assert(output==raw);
    double lz_us=median_us(lz,packed,output),rle_us=median_us(old,rle,output);
    std::printf("{\"lossless\":true,\"malformed_streams\":20000,\"decode_median_us\":%.3f,\"old_rle_median_us\":%.3f}\n",lz_us,rle_us);
}
