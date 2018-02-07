#pragma once
#include <stdint.h>
#ifdef __cplusplus
extern "C" {
#endif

#define FID_RESULT_OK 0
#define FID_RESULT_INVALIDARGUMENT -1
#define FID_RESULT_TIMESTAMPOVERFLOW -2
#define FID_RESULT_SEQUENCEOVERFLOW -3
#define FID_RESULT_GENERATOROVERFLOW -4
#define FID_RESULT_SYSTIMEISINPAST -5
#define FID_RESULT_WRONGSLICESIZE -6
#define FID_RESULT_BASE64DECODEERROR -7
#define FID_RESULT_BUFFERWRONGSIZE -8

typedef uint64_t FID;
typedef void *FID_GENERATOR;

int32_t flowerid_new(FID *self, uint64_t timestamp, uint64_t sequence, uint64_t generator);
int32_t flowerid_to_bytes(FID self, uint8_t *buffer, size_t buffer_size);
int32_t flowerid_from_bytes(FID *self, const uint8_t *buffer, size_t buffer_size);
int32_t flowerid_to_string(FID self, char *buffer, size_t buffer_size);
int32_t flowerid_from_string(FID *self, const char *buffer);
uint64_t flowerid_get_timestamp(FID self);
uint64_t flowerid_get_sequence(FID self);
uint64_t flowerid_get_generator(FID self);

int32_t flowerid_generator_new(FID_GENERATOR *self, uint64_t generator, int32_t wait_sequence);
int32_t flowerid_generator_new_ex(FID_GENERATOR *self, uint64_t generator, int64_t timestamp_offset, uint64_t timestamp_last, uint64_t sequence, int32_t wait_sequence, int32_t timestamp_in_seconds);
int32_t flowerid_generator_next(FID_GENERATOR self, FID *dst);
int32_t flowerid_generator_release(FID_GENERATOR self);

#ifdef __cplusplus
}
#endif
