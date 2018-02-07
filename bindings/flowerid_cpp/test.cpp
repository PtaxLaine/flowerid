#include <flowerid.h>
#include <iostream>
#include <ctime>
#include <gtest/gtest.h>

TEST(FID, fid_test)
{
    using namespace fid;
    uint64_t timestamp = 3020801146913;
    uint64_t sequence = 37;
    uint64_t generator = 160;
    uint64_t integer = 6335079166850929824;
    auto bin = "W\xea\xb8\xf0\x04 \x94\xa0";
    auto str = "V-q48AQglKA";
    std::array<uint8_t, 8> binvec = {0x57, 0xea, 0xb8, 0xf0, 0x04, 0x20, 0x94, 0xa0};

    auto fid = FID(timestamp, sequence, generator);
    ASSERT_EQ(fid.timestamp(), timestamp);
    ASSERT_EQ(fid.sequence(), sequence);
    ASSERT_EQ(fid.generator(), generator);
    ASSERT_EQ(fid.to_string(), str);
    ASSERT_EQ(memcmp(fid.to_bytes().data(), bin, 8), 0);
    ASSERT_EQ(fid.to_int(), integer);
    ASSERT_EQ(fid, FID::from_string(str));
    ASSERT_EQ(fid, FID::from_bytes(binvec));
    ASSERT_EQ(fid, FID::from_int(integer));

    ASSERT_TRUE(FID::from_bytes(binvec) == FID::from_int(integer));
    ASSERT_FALSE(FID::from_bytes(binvec) != FID::from_int(integer));
    ASSERT_TRUE(FID::from_bytes(binvec) < FID::from_int(integer + 1));
    ASSERT_TRUE(FID::from_bytes(binvec) <= FID::from_int(integer + 1));
    ASSERT_TRUE(FID::from_bytes(binvec) <= FID::from_int(integer));
    ASSERT_TRUE(FID::from_bytes(binvec) > FID::from_int(integer - 1));
    ASSERT_TRUE(FID::from_bytes(binvec) >= FID::from_int(integer - 1));
    ASSERT_TRUE(FID::from_bytes(binvec) >= FID::from_int(integer));

    std::stringstream fmt_test;
    fmt_test << fid;
    ASSERT_EQ(fmt_test.str(), str);
}

TEST(FID, fid_builder_defaults_test)
{
    using namespace fid;
    auto gen = FIDGeneratorBuilder(0);
    ASSERT_EQ(gen.get_generator(), 0);
    ASSERT_EQ(gen.get_timestamp_offset(), -1483228800);
    ASSERT_EQ(gen.get_timestamp_last(), 0);
    ASSERT_EQ(gen.get_sequence(), 0);
    ASSERT_EQ(gen.get_wait_sequence(), true);
    ASSERT_EQ(gen.get_timestamp_in_seconds(), false);
}

TEST(FID, fid_builder_test)
{
    using namespace fid;
    uint64_t generator = 0x8abf5b2ee9429cd6;
    auto timestamp_offset = -787943;
    uint64_t timestamp_last = 0xf32f3e5db6007b25;
    uint64_t sequence = 0x2bd043917317aaba;

    auto gen = FIDGeneratorBuilder(generator)
                   .timestamp_offset(timestamp_offset)
                   .timestamp_last(timestamp_last)
                   .sequence(sequence);
    ASSERT_EQ(gen.get_generator(), generator);
    ASSERT_EQ(gen.get_timestamp_offset(), timestamp_offset);
    ASSERT_EQ(gen.get_timestamp_last(), timestamp_last);
    ASSERT_EQ(gen.get_sequence(), sequence);

    ASSERT_FALSE(gen.not_wait_sequence().get_wait_sequence());
    ASSERT_TRUE(gen.wait_sequence().get_wait_sequence());

    ASSERT_TRUE(gen.timestamp_in_seconds().get_timestamp_in_seconds());
    ASSERT_FALSE(gen.timestamp_in_millisecond().get_timestamp_in_seconds());
}

TEST(FID, fid_generator_test)
{
    using namespace fid;
    uint64_t generator_id = 160;
    auto gen = FIDGeneratorBuilder(generator_id).timestamp_offset(-1483228800).timestamp_in_seconds().build();
    uint64_t timestamp = std::time(nullptr) - 1483228800;
    auto fid = gen.next();
    ASSERT_GE(fid.timestamp(), timestamp - 2);
    ASSERT_LE(fid.timestamp(), timestamp + 2);
    ASSERT_EQ(fid.sequence(), 0);
    ASSERT_EQ(fid.generator(), generator_id);
}

TEST(FID, fid_generator_overflow_test)
{
    using namespace fid;
    uint64_t generator_id = 160;
    auto gen = FIDGeneratorBuilder(generator_id).timestamp_offset(-1483228800).not_wait_sequence().timestamp_in_seconds().build();
    auto start_timestamp = gen.next().timestamp();
    while (1)
    {
        try
        {
            if (gen.next().timestamp() != start_timestamp)
            {
                break;
            }
        }
        catch (...)
        {
        }
    }
    ASSERT_THROW({
        for (size_t i = 1; i < 0xffffffff; ++i)
        {
            auto fid = gen.next();
            ASSERT_EQ(fid.sequence(), i);
            ASSERT_EQ(fid.generator(), generator_id);
            ASSERT_EQ(fid.timestamp(), start_timestamp + 1);
        }
    },
                 Error);
}
