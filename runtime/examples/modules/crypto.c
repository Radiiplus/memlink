#include <stdint.h>
#include <string.h>
#include <stdio.h>

static uint32_t fnv1a_hash(const unsigned char* data, size_t len) {
    uint32_t hash = 2166136261u;
    for (size_t i = 0; i < len; i++) {
        hash ^= data[i];
        hash *= 16777619u;
    }
    return hash;
}

__attribute__((visibility("default")))
int memlink_init(const unsigned char* config, unsigned long config_len) {
    (void)config;
    (void)config_len;
    return 0;
}

__attribute__((visibility("default")))
int memlink_call(unsigned int method_id, const unsigned char* args,
                unsigned long args_len, unsigned char* output) {
    (void)method_id;

    uint32_t hash = 0;
    if (args_len > 0 && args != NULL) {
        hash = fnv1a_hash(args, args_len);
    }

    sprintf((char*)output, "0x%08X", hash);
    return 0;
}

__attribute__((visibility("default")))
int memlink_shutdown(void) {
    return 0;
}
