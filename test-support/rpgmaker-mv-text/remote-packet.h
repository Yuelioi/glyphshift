#pragma once
#include <cstdint>
// Test transport only: all offsets are within one caller-owned allocation.
// No serialized pointers, filenames, PIDs or native handles in tracked fixtures.
struct RemotePacket {
    uint32_t size, operation, firstLength, secondLength, capacity, written, status;
};
constexpr uint32_t packetLimit = 4 * 1024 * 1024;
static_assert(sizeof(RemotePacket) == 28);
