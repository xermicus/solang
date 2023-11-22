#include <stdint.h>

#include "polkavm_guest.h"

POLKAVM_IMPORT(void, terminate, 12);
extern void __attribute__ ((naked)) terminate() { 
    __asm__( 
        ".word 0x0000000b\n" 
        ".word __polkavm_import_terminate\n" 
        "ret\n" 
        : 
        : 
        : "memory" 
    ); 
}

POLKAVM_IMPORT(uint32_t, set_storage, 1, uint32_t, uint32_t, uint32_t, uint32_t);
extern uint32_t __attribute__ ((naked)) set_storage(uint8_t *key_ptr, uint32_t key_len, uint8_t *value_ptr, uint32_t value_len) { 
    __asm__( 
        ".word 0x0000000b\n" 
        ".word __polkavm_import_set_storage\n" 
        "ret\n" 
        : 
        : 
        : "memory" 
    ); 
}

POLKAVM_IMPORT(uint32_t, get_storage, 3, uint32_t, uint32_t, uint32_t, uint32_t);
extern uint32_t __attribute__ ((naked)) get_storage(uint8_t *key_ptr, uint32_t key_len, uint8_t *out_ptr, uint32_t *out_len_ptr) { 
    __asm__( 
        ".word 0x0000000b\n" 
        ".word __polkavm_import_get_storage\n" 
        "ret\n" 
        : 
        : 
        : "memory" 
    ); 
}

POLKAVM_IMPORT(void, input, 13, uint32_t, uint32_t);
extern void __attribute__ ((naked)) input(uint8_t *out_ptr, uint32_t *out_len_ptr) { 
    __asm__( 
        ".word 0x0000000b\n" 
        ".word __polkavm_import_input\n" 
        "ret\n" 
        : 
        : 
        : "memory" 
    ); 
}

POLKAVM_IMPORT(void, seal_return, 14, uint32_t, uint32_t, uint32_t);
extern void __attribute__ ((naked)) seal_return(uint32_t flags, uint8_t *data_ptr, uint32_t data_len) { 
    __asm__( 
        ".word 0x0000000b\n" 
        ".word __polkavm_import_seal_return\n" 
        "ret\n" 
        : 
        : 
        : "memory" 
    ); 
}

POLKAVM_IMPORT(void, value_transferred, 27, uint32_t, uint32_t);
extern void __attribute__ ((naked)) value_transferred(uint8_t *out_ptr, uint32_t *out_len_ptr) { 
    __asm__( 
        ".word 0x0000000b\n" 
        ".word __polkavm_import_value_transferred\n" 
        "ret\n" 
        : 
        : 
        : "memory" 
    ); 
}


POLKAVM_IMPORT(uint32_t, debug_message, 37, uint32_t, uint32_t);
extern uint32_t __attribute__ ((naked)) debug_message(uint8_t *str_ptr, uint32_t str_len) { 
    __asm__( 
        ".word 0x0000000b\n" 
        ".word __polkavm_import_debug_message\n" 
        "ret\n" 
        : 
        : 
        : "memory" 
    ); 
}