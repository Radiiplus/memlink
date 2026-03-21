#include <stdint.h>
#include <string.h>
#include <ctype.h>
#include <stdio.h>

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

    if (args_len == 0 || args == NULL) {
        return -1;
    }

    size_t len = args_len < 4095 ? args_len : 4095;
    memcpy(output, args, len);
    output[len] = '\0';

    for (size_t i = 0; i < len; i++) {
        output[i] = toupper(output[i]);
    }

    return 0;
}

__attribute__((visibility("default")))
int memlink_shutdown(void) {
    return 0;
}
