// Decode the embedded DN-LZ1 sprite stream into a caller-owned pixel buffer.
// Every literal/copy checks input and output bounds before writing.

#pragma once
#include <cstddef>
#include <cstdint>
#include <cstring>

namespace damage_numbers {
// DN-LZ1: literals (control 0..127, length control+1), or overlapping copies
// (128..254, length control-125; 255, length 130+LE16), followed by LE16
// backward distance. No allocation, scratch buffer, or per-frame decoding.
// Keep one callable implementation for the exact compiled ARM bounds fixture.
inline __attribute__((noinline)) bool decode_sprite(const uint8_t *source, size_t source_size,
                                                    uint8_t *pixels, size_t pixel_bytes) {
    if (!source || !pixels)
        return false;
    size_t at = 0, written = 0;
    while (at < source_size) {
        const unsigned control = source[at++];
        if (control < 128) {
            const size_t length = control + 1;
            if (length > source_size - at || length > pixel_bytes - written)
                return false;
            for (size_t i = 0; i < length; ++i)
                pixels[written++] = source[at++];
        } else {
            size_t length = control - 125;
            if (control == 255) {
                if (source_size - at < 2)
                    return false;
                length = 130 + unsigned(source[at]) + unsigned(source[at + 1]) * 256;
                at += 2;
            }
            if (source_size - at < 2)
                return false;
            const size_t distance = unsigned(source[at]) + unsigned(source[at + 1]) * 256;
            at += 2;
            if (!distance || distance > written || length > pixel_bytes - written)
                return false;
            if (distance == 1) {
                std::memset(pixels + written, pixels[written - 1], length);
                written += length;
                continue;
            }
            // Four-byte copies are safe even when the overall runs overlap:
            // each source word is fully initialized if distance is at least 4.
            if (distance >= 4)
                while (length >= 4) {
                    uint32_t word;
                    std::memcpy(&word, pixels + written - distance, 4);
                    std::memcpy(pixels + written, &word, 4);
                    written += 4;
                    length -= 4;
                }
            for (size_t i = 0; i < length; ++i) {
                pixels[written] = pixels[written - distance];
                ++written;
            }
        }
    }
    return written == pixel_bytes;
}
} // namespace damage_numbers
