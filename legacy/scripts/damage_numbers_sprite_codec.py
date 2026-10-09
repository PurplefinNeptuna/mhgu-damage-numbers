# Build-time DN-LZ1 encoder and independent decoder. Prepared sprite headers
# carry the compressed bytes; audits compare the decoded pixels with source RGBA.

from paths import MOD, SCRIPTS, FORGE, DEPS, PYTHON, CONTAINER_STORAGE, input_path
"""Owned DN-LZ1 asset codec. Runtime format is documented in sprite_codec.h."""
from collections import defaultdict, deque


# Emit bounded literal runs or backward matches using the DN-LZ1 control format.
# Copies may overlap, so the decoder can reproduce long repeated patterns.
def encode(source):
    output = bytearray()
    recent = defaultdict(lambda: deque(maxlen=8))
    literals = bytearray()
    at = 0
    while at < len(source):
        key = source[at:at + 3]
        best, distance = 0, 0
        if len(key) == 3:
            for previous in reversed(recent[key]):
                if at - previous > 65535:
                    break
                length = 3
                while length < min(4096, len(source) - at) and source[previous + length] == source[at + length]:
                    length += 1
                if length > best:
                    best, distance = length, at - previous
                if length == 4096:
                    break
        if best >= 3:
            if literals:
                output.append(len(literals) - 1)
                output.extend(literals)
                literals.clear()
            if best >= 130:
                output.extend((255, (best - 130) & 255, (best - 130) >> 8))
            else:
                output.append(125 + best)
            output.extend((distance & 255, distance >> 8))
            end = at + best
        else:
            literals.append(source[at])
            end = at + 1
            if len(literals) == 128:
                output.append(127)
                output.extend(literals)
                literals.clear()
        while at < end:
            recent[source[at:at + 3]].append(at)
            at += 1
    if literals:
        output.append(len(literals) - 1)
        output.extend(literals)
    return bytes(output)


def decode(source, size):
    """Independent reference decoder used for build and payload audits."""
    output = bytearray()
    at = 0
    while at < len(source):
        control = source[at]
        at += 1
        if control < 128:
            length = control + 1
            if length > len(source) - at or length > size - len(output):
                raise ValueError('Invalid literal extent')
            output.extend(source[at:at + length])
            at += length
        else:
            length = control - 125
            if control == 255:
                if len(source) - at < 2:
                    raise ValueError('Truncated extended length')
                length = 130 + int.from_bytes(source[at:at + 2], 'little')
                at += 2
            if len(source) - at < 2:
                raise ValueError('Truncated copy distance')
            distance = int.from_bytes(source[at:at + 2], 'little')
            at += 2
            if not distance or distance > len(output) or length > size - len(output):
                raise ValueError('Invalid copy extent')
            for _ in range(length):
                output.append(output[-distance])
    if len(output) != size:
        raise ValueError('Incomplete image')
    return bytes(output)
