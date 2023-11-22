#include <stdint.h>

#include "polkavm_guest.h"

uint32_t __attribute__ ((naked)) set_storage(uint8_t *key_ptr, uint32_t key_len, uint8_t *value_ptr, uint32_t value_len) 
    POLKAVM_ECALLI_TRAMPOLINE(uint32_t, set_storage, 1, uint32_t, uint32_t, uint32_t, uint32_t)

uint32_t __attribute__ ((naked)) get_storage(uint8_t *key_ptr, uint32_t key_len, uint8_t *out_ptr, uint32_t *out_len_ptr) 
    POLKAVM_ECALLI_TRAMPOLINE(uint32_t, get_storage, 3, uint32_t, uint32_t, uint32_t, uint32_t)

uint32_t __attribute__ ((naked)) seal_call(uint8_t *ptr)
    POLKAVM_ECALLI_TRAMPOLINE(void, seal_call, 7, uint32_t)

void __attribute__ ((naked)) terminate()
    POLKAVM_ECALLI_TRAMPOLINE(void, terminate, 12)

void __attribute__ ((naked)) input(uint8_t *out_ptr, uint32_t *out_len_ptr) 
    POLKAVM_ECALLI_TRAMPOLINE(void, input, 13, uint32_t, uint32_t)

void __attribute__ ((naked)) seal_return(uint32_t flags, uint8_t *data_ptr, uint32_t data_len)
    POLKAVM_ECALLI_TRAMPOLINE(void, seal_return, 14, uint32_t, uint32_t, uint32_t)

void value_transferred(uint8_t *out_ptr, uint32_t *out_len_ptr) 
    POLKAVM_ECALLI_TRAMPOLINE(void, value_transferred, 27, uint32_t, uint32_t)

void __attribute__ ((naked)) hash_keccak_256(uint8_t *input_ptr, uint32_t input_len, uint8_t *out_ptr)
    POLKAVM_ECALLI_TRAMPOLINE(void, hash_keccak_256, 33, uint32_t, uint32_t, uint32_t)

uint32_t __attribute__ ((naked)) debug_message(uint8_t *str_ptr, uint32_t str_len) 
    POLKAVM_ECALLI_TRAMPOLINE(uint32_t, debug_message, 37, uint32_t, uint32_t)