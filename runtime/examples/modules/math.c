#include <stdint.h>
#include <string.h>
#include <stdio.h>
#include <stdlib.h>

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

    int a = 0, b = 0;
    if (args_len > 0 && args != NULL) {
        char buffer[64] = {0};
        size_t len = args_len < 63 ? args_len : 63;
        memcpy(buffer, args, len);
        buffer[len] = '\0';

        char* comma = strchr(buffer, ',');
        if (comma != NULL) {
            *comma = '\0';
            a = atoi(buffer);
            b = atoi(comma + 1);
        }
    }

    sprintf((char*)output, "%d", a + b);
    return 0;
}

__attribute__((visibility("default")))
int memlink_shutdown(void) {
    return 0;
}
