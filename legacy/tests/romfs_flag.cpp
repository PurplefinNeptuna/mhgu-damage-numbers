// SPDX-FileCopyrightText: Copyright 2018 yuzu Emulator Project
// SPDX-License-Identifier: GPL-2.0-or-later
// Test-only model of Eden d16735f5b6's RomFS layout/sort/concatenation logic.
// See scripts/test_damage_numbers_romfs_flag.py for source provenance and limits.
#include <algorithm>
#include <array>
#include <cstdint>
#include <fstream>
#include <iostream>
#include <string>
#include <vector>

struct Entry { uint64_t offset, size; std::string path; };
int main(int argc, char** argv) {
    if (argc != 3) return 2;
    std::vector<Entry> files;
    std::ifstream input(argv[1]);
    uint64_t size;
    std::string path;
    while (input >> size) {
        input.get(); std::getline(input, path); files.push_back({0,size,path});
    }
    std::array<unsigned char,16> expected{};
    std::ifstream font_file(argv[2],std::ios::binary);
    if (!font_file.read(reinterpret_cast<char*>(expected.data()),expected.size())) return 3;
    const std::string flag="nativeNX/forge/damage-numbers-capture.enabled";
    const std::string font="nativeNX/forge/fonts/Roboto-Medium.ttf";
    std::sort(files.begin(),files.end(),[](const Entry& a,const Entry& b) {return a.path < b.path;});
    for (int flag_bytes : {-1,0,1,2,8}) {
        auto output=files;
        if (flag_bytes<0)
            output.erase(std::remove_if(output.begin(),output.end(),[&](const Entry& e) {return e.path==flag;}),output.end());
        uint64_t offset=0,font_offset=0;
        for (auto& e:output) {
            if (e.path==flag) e.size=unsigned(flag_bytes);
            offset=(offset+15)&~uint64_t(15); e.offset=0x200+offset; offset+=e.size;
            if (e.path==font) font_offset=e.offset;
        }
        if (!font_offset) return 4;
        output.insert(output.begin(),{0,0x50,"header"});
        output.push_back({(0x200+offset+3)&~uint64_t(3),0x100,"metadata"});
        // This offset-only std::sort is intentionally unstable, as upstream.
        std::sort(output.begin(),output.end(),[](const Entry& a,const Entry& b) {return a.offset < b.offset;});
        std::vector<Entry> concatenation;
        uint64_t last=0;
        for (const auto& e:output) {
            if (e.offset>last) concatenation.push_back({last,e.offset-last,"padding"});
            concatenation.push_back(e); last=e.offset+e.size;
        }
        auto found=std::prev(std::upper_bound(concatenation.begin(),concatenation.end(),font_offset,
            [](uint64_t key,const Entry& e) {return key<e.offset;}));
        std::array<unsigned char,16> actual{};
        if (found->path==font && found->offset==font_offset && found->size>=actual.size()) actual=expected;
        const bool good=actual==expected;
        if (good != (flag_bytes!=0)) {
            std::cerr<<"Unexpected host sort outcome; fixture did not reproduce the observed layout\n";
            return 5;
        }
        std::cout<<"{\"flag_bytes\":"<<flag_bytes<<",\"font_read_ok\":"<<(good?"true":"false")
                 <<",\"selected\":\""<<found->path<<"\",\"font_offset\":"<<font_offset<<"}\n";
    }
}
