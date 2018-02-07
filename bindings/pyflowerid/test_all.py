import time
import unittest
import pyflowerid as pyfid


class TestFID(unittest.TestCase):
    timestamp = 3020801146913
    sequence = 37
    generator = 160
    integer = 6335079166850929824
    binstr = b"W\xea\xb8\xf0\x04 \x94\xa0"
    strstr = "V-q48AQglKA"

    def test_new(self):
        fid = pyfid.FId(TestFID.timestamp, TestFID.sequence, TestFID.generator)
        self.assertEqual(fid.timestamp, TestFID.timestamp)
        self.assertEqual(fid.sequence, TestFID.sequence)
        self.assertEqual(fid.generator, TestFID.generator)
        self.assertEqual(int(fid), TestFID.integer)
        self.assertEqual(str(fid), TestFID.strstr)
        self.assertEqual(fid, pyfid.FId(TestFID.integer))
        self.assertEqual(fid, pyfid.FId(TestFID.strstr))
        self.assertEqual(fid, pyfid.FId(TestFID.binstr))

        self.assertEqual(pyfid.FId(TestFID.integer),
                         pyfid.FId(TestFID.integer))
        self.assertGreaterEqual(
            pyfid.FId(TestFID.integer), pyfid.FId(TestFID.integer))
        self.assertLessEqual(pyfid.FId(TestFID.integer),
                             pyfid.FId(TestFID.integer))
        self.assertGreater(pyfid.FId(TestFID.integer),
                           pyfid.FId(TestFID.integer - 1))
        self.assertLess(pyfid.FId(TestFID.integer),
                        pyfid.FId(TestFID.integer + 1))


class TestBuilder(unittest.TestCase):
    timestamp_offset = -78798423
    generator = 160
    timestamp_last = 798464641
    sequence = 879749123

    def test_builder_defaults(self):
        gen = pyfid.FidGeneratorBuilder(0)
        self.assertEqual(gen.get_generator(), 0)
        self.assertEqual(gen.get_timestamp_offset(), -1483228800)
        self.assertEqual(gen.get_timestamp_last(), 0)
        self.assertEqual(gen.get_sequence(), 0)
        self.assertEqual(gen.get_wait_sequence(), 1)
        self.assertEqual(gen.get_timestamp_in_seconds(), 0)

    def test_builder(self):
        gen = pyfid.FidGeneratorBuilder(TestBuilder.generator) \
            .timestamp_offset(TestBuilder.timestamp_offset) \
            .timestamp_last(TestBuilder.timestamp_last) \
            .sequence(TestBuilder.sequence)
        self.assertEqual(gen.get_generator(), TestBuilder.generator)
        self.assertEqual(gen.get_timestamp_offset(),
                         TestBuilder.timestamp_offset)
        self.assertEqual(gen.get_timestamp_last(), TestBuilder.timestamp_last)
        self.assertEqual(gen.get_sequence(), TestBuilder.sequence)
        gen.not_wait_sequence()
        self.assertEqual(gen.get_wait_sequence(), 0)
        gen.wait_sequence()
        self.assertEqual(gen.get_wait_sequence(), 1)
        gen.timestamp_in_seconds()
        self.assertEqual(gen.get_timestamp_in_seconds(), 1)
        gen.timestamp_in_millisecond()
        self.assertEqual(gen.get_timestamp_in_seconds(), 0)


class TestGenerator(unittest.TestCase):
    def test_generator(self):
        gen = pyfid.FidGeneratorBuilder(TestBuilder.generator) \
            .timestamp_offset(0) \
            .timestamp_in_seconds() \
            .build()
        timer = int(time.time())
        fid = gen.next()
        self.assertEqual(fid.generator, TestBuilder.generator)
        self.assertEqual(fid.sequence, 0)
        self.assertGreaterEqual(int(fid.timestamp), timer)
        self.assertLessEqual(int(fid.timestamp), timer + 1)

    def test_generator_overflow(self):
        gen = pyfid.FidGeneratorBuilder(TestBuilder.generator) \
            .timestamp_offset(0) \
            .timestamp_in_seconds() \
            .not_wait_sequence() \
            .build()
        start_timestamp = int(time.time())
        while 1:
            try:
                if gen.next().timestamp != start_timestamp:
                    break
            except:
                pass
        with self.assertRaises(pyfid.Error):
            for i in range(1, 0xffffffffff):
                fid = gen.next()
                self.assertEqual(fid.sequence, i)
                self.assertEqual(fid.generator, TestBuilder.generator)
                self.assertEqual(fid.timestamp, start_timestamp + 1)

if __name__ == '__main__':
    unittest.main()
