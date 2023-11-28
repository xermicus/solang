#include <stdint.h>

#include "polkavm_guest.h"

uint32_t __attribute__ ((naked)) set_storage(uint8_t *key_ptr, uint32_t key_len, uint8_t *value_ptr, uint32_t value_len) 
    POLKAVM_ECALLI_TRAMPOLINE(uint32_t, set_storage, 1, uint32_t, uint32_t, uint32_t, uint32_t)

uint32_t __attribute__ ((naked)) clear_storage(uint8_t *key_ptr, uint32_t key_len) 
    POLKAVM_ECALLI_TRAMPOLINE(uint32_t, clear_storage, 2, uint32_t, uint32_t)

uint32_t __attribute__ ((naked)) get_storage(uint8_t *key_ptr, uint32_t key_len, uint8_t *out_ptr, uint32_t *out_len_ptr) 
    POLKAVM_ECALLI_TRAMPOLINE(uint32_t, get_storage, 3, uint32_t, uint32_t, uint32_t, uint32_t)

uint32_t __attribute__ ((naked)) contains_storage(uint8_t *key_ptr, uint32_t key_len) 
    POLKAVM_ECALLI_TRAMPOLINE(uint32_t, contains_storage, 4, uint32_t, uint32_t)

uint32_t __attribute__ ((naked)) take_storage(uint8_t *key_ptr, uint32_t key_len, uint8_t *out_ptr, uint32_t *out_len_ptr) 
    POLKAVM_ECALLI_TRAMPOLINE(uint32_t, take_storage, 5, uint32_t, uint32_t, uint32_t, uint32_t)

uint32_t __attribute__ ((naked)) seal_call(uint8_t *ptr)
    POLKAVM_ECALLI_TRAMPOLINE(void, seal_call, 7, uint32_t)

uint32_t __attribute__ ((naked)) delegate_call(
    uint32_t flags,
    uint8_t *code_hash_ptr,
    uint8_t *input_data_ptr,
    uint32_t input_data_len,
    uint8_t *out_ptr,
    uint32_t *out_len_ptr) 
    POLKAVM_ECALLI_TRAMPOLINE(uint32_t, delegate_call, 9, uint32_t, uint32_t, uint32_t, uint32_t, uint32_t, uint32_t)

uint32_t __attribute__ ((naked)) instantiate(uint8_t *ptr)
    POLKAVM_ECALLI_TRAMPOLINE(uint32_t, instantiate, 10, uint32_t)

void __attribute__ ((naked)) terminate(uint8_t *beneficary_ptr)
    POLKAVM_ECALLI_TRAMPOLINE(void, terminate, 12, uint32_t)

void __attribute__ ((naked)) input(uint8_t *out_ptr, uint32_t *out_len_ptr) 
    POLKAVM_ECALLI_TRAMPOLINE(void, input, 13, uint32_t, uint32_t)

void __attribute__ ((naked)) seal_return(uint32_t flags, uint8_t *data_ptr, uint32_t data_len)
    POLKAVM_ECALLI_TRAMPOLINE(void, seal_return, 14, uint32_t, uint32_t, uint32_t)

void __attribute__ ((naked)) caller(uint8_t *out_ptr, uint32_t *out_len_ptr)
    POLKAVM_ECALLI_TRAMPOLINE(void, caller, 15, uint32_t, uint32_t)

uint32_t __attribute__ ((naked)) is_contract(uint8_t *ptr)
    POLKAVM_ECALLI_TRAMPOLINE(uint32_t, is_contract, 16, uint32_t)

uint32_t __attribute__ ((naked)) code_hash(uint8_t *account_ptr, uint8_t *out_ptr, uint32_t *out_len_ptr)
    POLKAVM_ECALLI_TRAMPOLINE(uint32_t, code_hash, 17, uint32_t, uint32_t, uint32_t)

void __attribute__ ((naked)) own_code_hash(uint8_t *out_ptr, uint32_t *out_len_ptr)
    POLKAVM_ECALLI_TRAMPOLINE(void, own_code_hash, 18, uint32_t, uint32_t)

uint32_t __attribute__ ((naked)) caller_is_origin()
    POLKAVM_ECALLI_TRAMPOLINE(uint32_t, caller_is_origin, 19)

uint32_t __attribute__ ((naked)) caller_is_root()
    POLKAVM_ECALLI_TRAMPOLINE(uint32_t, caller_is_root, 20)

void __attribute__ ((naked)) address(uint8_t *out_ptr, uint32_t *out_len_ptr)
    POLKAVM_ECALLI_TRAMPOLINE(void, address, 21, uint32_t, uint32_t)

void __attribute__ ((naked)) weight_to_fee(uint64_t gas, uint8_t *out_ptr, uint32_t *out_len_ptr)
    POLKAVM_ECALLI_TRAMPOLINE(void, weight_to_fee, 22, uint64_t, uint32_t, uint32_t)

void __attribute__ ((naked)) gas_left(uint8_t *out_ptr, uint32_t *out_len_ptr)
    POLKAVM_ECALLI_TRAMPOLINE(void, gas_left, 24, uint32_t, uint32_t)

void __attribute__ ((naked)) balance(uint8_t *out_ptr, uint32_t *out_len_ptr) 
    POLKAVM_ECALLI_TRAMPOLINE(void, balance, 26, uint32_t, uint32_t)

void __attribute__ ((naked)) value_transferred(uint8_t *out_ptr, uint32_t *out_len_ptr) 
    POLKAVM_ECALLI_TRAMPOLINE(void, value_transferred, 27, uint32_t, uint32_t)

void __attribute__ ((naked)) now(uint8_t *out_ptr, uint32_t *out_len_ptr) 
    POLKAVM_ECALLI_TRAMPOLINE(void, now, 28, uint32_t, uint32_t)

void __attribute__ ((naked)) minimum_balance(uint8_t *out_ptr, uint32_t *out_len_ptr) 
    POLKAVM_ECALLI_TRAMPOLINE(void, minimum_balance, 29, uint32_t, uint32_t)

void __attribute__ ((naked)) deposit_event(uint8_t *topics_ptr, uint32_t topics_len, uint8_t *data_ptr, uint32_t data_len) 
    POLKAVM_ECALLI_TRAMPOLINE(void, deposit_event, 30, uint32_t, uint32_t, uint32_t, uint32_t)

void __attribute__ ((naked)) block_number(uint8_t *out_ptr, uint32_t *out_len_ptr) 
    POLKAVM_ECALLI_TRAMPOLINE(void, block_number, 31, uint32_t, uint32_t)

void __attribute__ ((naked)) hash_sha2_256(uint8_t *input_ptr, uint32_t input_len, uint8_t *out_ptr)
    POLKAVM_ECALLI_TRAMPOLINE(void, hash_sha2_256, 32, uint32_t, uint32_t, uint32_t)

void __attribute__ ((naked)) hash_keccak_256(uint8_t *input_ptr, uint32_t input_len, uint8_t *out_ptr)
    POLKAVM_ECALLI_TRAMPOLINE(void, hash_keccak_256, 33, uint32_t, uint32_t, uint32_t)

void __attribute__ ((naked)) hash_blake2_256(uint8_t *input_ptr, uint32_t input_len, uint8_t *out_ptr)
    POLKAVM_ECALLI_TRAMPOLINE(void, hash_blake2_256, 34, uint32_t, uint32_t, uint32_t)

void __attribute__ ((naked)) hash_blake2_128(uint8_t *input_ptr, uint32_t input_len, uint8_t *out_ptr)
    POLKAVM_ECALLI_TRAMPOLINE(void, hash_blake2_128, 35, uint32_t, uint32_t, uint32_t)

uint32_t __attribute__ ((naked)) call_chain_extension(
    uint32_t id,
    uint8_t *input_ptr,
    uint32_t input_len,
    uint8_t *out_ptr,
    uint32_t *out_len_ptr) 
    POLKAVM_ECALLI_TRAMPOLINE(uint32_t, call_chain_extension, 36, uint32_t, uint32_t, uint32_t, uint32_t, uint32_t)

uint32_t __attribute__ ((naked)) debug_message(uint8_t *str_ptr, uint32_t str_len) 
    POLKAVM_ECALLI_TRAMPOLINE(uint32_t, debug_message, 37, uint32_t, uint32_t)

uint32_t __attribute__ ((naked)) set_code_hash(uint8_t *code_hash_ptr) 
    POLKAVM_ECALLI_TRAMPOLINE(uint32_t, set_code_hash, 41, uint32_t)

uint64_t __attribute__ ((naked)) instantiation_nonce()
    POLKAVM_ECALLI_TRAMPOLINE(void, instantiation_nonce, 45)

uint32_t __attribute__ ((naked)) transfer(uint8_t *account_ptr, uint32_t account_len, uint8_t *value_ptr, uint32_t value_len) 
    POLKAVM_ECALLI_TRAMPOLINE(uint32_t, transfer, 6, uint32_t, uint32_t, uint32_t, uint32_t)
