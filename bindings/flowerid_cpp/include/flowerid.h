#pragma once
#include <string>
#include <array>
#include <iostream>
#include <cinttypes>
#include <exception>

namespace fid
{
class Error : public std::exception
{
  public:
    Error(int32_t code);
    int32_t code() const;
    virtual const char *what() const noexcept override;

  private:
    int32_t _code;
};

class FID
{
    FID(uint64_t origin);

  public:
    FID(uint64_t timestamp, uint64_t sequence, uint64_t generator);
    uint64_t timestamp() const;
    uint64_t sequence() const;
    uint64_t generator() const;
    std::string to_string() const;
    std::array<uint8_t, 8> to_bytes() const;
    uint64_t to_int() const;
    static FID from_string(const std::string &origin);
    static FID from_bytes(const std::array<uint8_t, 8> &origin);
    static FID from_int(uint64_t origin);

    bool operator>(const FID &right)const;
    bool operator<(const FID &right)const;
    bool operator>=(const FID &right)const;
    bool operator<=(const FID &right)const;
    bool operator==(const FID &right)const;
    bool operator!=(const FID &right)const;

  private:
    uint64_t _fid;
};

std::ostream &operator<<(std::ostream &os, const FID &dt);

class FIDGenerator;
class FIDGeneratorBuilder
{
  public:
    FIDGeneratorBuilder(uint64_t generator);
    FIDGeneratorBuilder &timestamp_offset(int64_t timestamp_offset);
    FIDGeneratorBuilder &timestamp_last(uint64_t timestamp_last);
    FIDGeneratorBuilder &sequence(uint64_t sequence);
    FIDGeneratorBuilder &wait_sequence();
    FIDGeneratorBuilder &not_wait_sequence();
    FIDGeneratorBuilder &timestamp_in_seconds();
    FIDGeneratorBuilder &timestamp_in_millisecond();
    FIDGenerator build() const;

    uint64_t get_generator() const;
    int64_t get_timestamp_offset() const;
    uint64_t get_timestamp_last() const;
    uint64_t get_sequence() const;
    bool get_wait_sequence() const;
    bool get_timestamp_in_seconds() const;

  private:
    uint64_t _generator;
    int64_t _timestamp_offset;
    uint64_t _timestamp_last;
    uint64_t _sequence;
    bool _wait_sequence;
    bool _timestamp_in_seconds;
};

class FIDGenerator
{
  public:
    FIDGenerator(const FIDGeneratorBuilder &cfg);
    ~FIDGenerator();
    FIDGenerator(const FIDGenerator &) = delete;
    FIDGenerator(FIDGenerator &&);
    FIDGenerator &operator=(const FIDGenerator &) = delete;
    FIDGenerator &operator=(FIDGenerator &&);
    FID next();

  public:
    void *_generator;
};
}
