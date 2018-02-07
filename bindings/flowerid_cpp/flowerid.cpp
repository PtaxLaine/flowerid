#include "flowerid.h"
#include "flowerid_ex.h"

namespace fid
{

std::ostream &operator<<(std::ostream &os, const FID &dt)
{
    os << dt.to_string();
    return os;
}

Error::Error(int32_t code)
    : _code(code)
{
}

const char *Error::what() const noexcept
{
    switch (this->_code)
    {
    case FID_RESULT_OK:
        return "no error";
    case FID_RESULT_INVALIDARGUMENT:
        return "invalid argument error";
    case FID_RESULT_TIMESTAMPOVERFLOW:
        return "timestamp overflow error";
    case FID_RESULT_SEQUENCEOVERFLOW:
        return "sequence overflow error";
    case FID_RESULT_GENERATOROVERFLOW:
        return "generator overflow error";
    case FID_RESULT_SYSTIMEISINPAST:
        return "system time is in past error";
    case FID_RESULT_WRONGSLICESIZE:
        return "wrong slice size error";
    case FID_RESULT_BASE64DECODEERROR:
        return "base64 decode error";
    case FID_RESULT_BUFFERWRONGSIZE:
        return "wrong buffer size error";
    default:
        if (this->_code > 0)
            return "no error but failed";
        else
            return "unknown error";
    }
}

int32_t Error::code() const
{
    return this->_code;
}

FID::FID(uint64_t origin)
    : _fid(origin)
{
}

FID::FID(uint64_t timestamp, uint64_t sequence, uint64_t generator)
{
    auto err = flowerid_new(&this->_fid, timestamp, sequence, generator);
    if (err < 0)
        throw Error(err);
}

uint64_t FID::timestamp() const
{
    return flowerid_get_timestamp(this->_fid);
}

uint64_t FID::sequence() const
{
    return flowerid_get_sequence(this->_fid);
}

uint64_t FID::generator() const
{
    return flowerid_get_generator(this->_fid);
}

std::string FID::to_string() const
{
    char buffer[12];
    auto res = flowerid_to_string(this->_fid, buffer, sizeof(buffer));
    if (res != 11)
        throw Error(res);
    return std::string(buffer);
}

std::array<uint8_t, 8> FID::to_bytes() const
{
    std::array<uint8_t, 8> vec;
    auto err = flowerid_to_bytes(this->_fid, vec.data(), vec.size());
    if (err != 8)
        throw Error(err);
    return vec;
}

uint64_t FID::to_int() const
{
    return this->_fid;
}

FID FID::from_string(const std::string &origin)
{
    ::FID fid;
    auto err = flowerid_from_string(&fid, origin.c_str());
    if (err != 0)
        throw Error(err);
    return FID(fid);
}

FID FID::from_bytes(const std::array<uint8_t, 8> &origin)
{
    ::FID fid;
    auto err = flowerid_from_bytes(&fid, origin.data(), origin.size());
    if (err != 0)
        throw Error(err);
    return FID(fid);
}

FID FID::from_int(uint64_t origin)
{
    return FID(origin);
}

bool FID::operator>(const FID &right)const { return this->_fid > right._fid; }
bool FID::operator<(const FID &right)const { return this->_fid < right._fid; }
bool FID::operator>=(const FID &right)const { return this->_fid >= right._fid; }
bool FID::operator<=(const FID &right)const { return this->_fid <= right._fid; }
bool FID::operator==(const FID &right)const { return this->_fid == right._fid; }
bool FID::operator!=(const FID &right)const { return this->_fid != right._fid; }

FIDGeneratorBuilder::FIDGeneratorBuilder(uint64_t generator)
    : _generator(generator),
      _timestamp_offset(-1483228800),
      _timestamp_last(0),
      _sequence(0),
      _wait_sequence(true),
      _timestamp_in_seconds(false)
{
}

FIDGeneratorBuilder &FIDGeneratorBuilder::timestamp_offset(int64_t timestamp_offset)
{
    this->_timestamp_offset = timestamp_offset;
    return *this;
}

FIDGeneratorBuilder &FIDGeneratorBuilder::timestamp_last(uint64_t timestamp_last)
{
    this->_timestamp_last = timestamp_last;
    return *this;
}

FIDGeneratorBuilder &FIDGeneratorBuilder::sequence(uint64_t sequence)
{
    this->_sequence = sequence;
    return *this;
}

FIDGeneratorBuilder &FIDGeneratorBuilder::wait_sequence()
{
    this->_wait_sequence = true;
    return *this;
}

FIDGeneratorBuilder &FIDGeneratorBuilder::not_wait_sequence()
{
    this->_wait_sequence = false;
    return *this;
}

FIDGeneratorBuilder &FIDGeneratorBuilder::timestamp_in_seconds()
{
    this->_timestamp_in_seconds = true;
    return *this;
}

FIDGeneratorBuilder &FIDGeneratorBuilder::timestamp_in_millisecond()
{
    this->_timestamp_in_seconds = false;
    return *this;
}

FIDGenerator FIDGeneratorBuilder::build() const
{
    return FIDGenerator(*this);
}

uint64_t FIDGeneratorBuilder::get_generator() const
{
    return this->_generator;
}

int64_t FIDGeneratorBuilder::get_timestamp_offset() const
{
    return this->_timestamp_offset;
}

uint64_t FIDGeneratorBuilder::get_timestamp_last() const
{
    return this->_timestamp_last;
}

uint64_t FIDGeneratorBuilder::get_sequence() const
{
    return this->_sequence;
}

bool FIDGeneratorBuilder::get_wait_sequence() const
{
    return this->_wait_sequence;
}

bool FIDGeneratorBuilder::get_timestamp_in_seconds() const
{
    return this->_timestamp_in_seconds;
}

FIDGenerator::FIDGenerator(const FIDGeneratorBuilder &cfg)
{
    auto err = flowerid_generator_new_ex(&this->_generator,
                                         cfg.get_generator(),
                                         cfg.get_timestamp_offset(),
                                         cfg.get_timestamp_last(),
                                         cfg.get_sequence(),
                                         cfg.get_wait_sequence(),
                                         cfg.get_timestamp_in_seconds());
    if (err < 0)
        throw Error(err);
}

FIDGenerator::~FIDGenerator()
{
    flowerid_generator_release(this->_generator);
}
FIDGenerator::FIDGenerator(FIDGenerator &&origin)
    : _generator(origin._generator)
{
    origin._generator = nullptr;
}

FIDGenerator &FIDGenerator::operator=(FIDGenerator &&origin)
{
    flowerid_generator_release(this->_generator);
    this->_generator = origin._generator;
    origin._generator = nullptr;
    return *this;
}

FID FIDGenerator::next()
{
    ::FID result;
    auto err = flowerid_generator_next(this->_generator, &result);
    if (err < 0)
        throw Error(err);
    return FID::from_int(result);
}
}
