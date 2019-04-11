from cpython cimport array


cdef extern from "stdint.h":
    ctypedef unsigned long long uint64_t
    ctypedef unsigned long uint32_t
    ctypedef unsigned char uint8_t
    ctypedef long long int64_t
    ctypedef long int32_t

	
cdef extern from "flowerid.h":
    ctypedef uint64_t FID
    ctypedef void* FID_GENERATOR

    cdef int FID_RESULT_OK
    cdef int FID_RESULT_INVALIDARGUMENT
    cdef int FID_RESULT_TIMESTAMPOVERFLOW
    cdef int FID_RESULT_SEQUENCEOVERFLOW
    cdef int FID_RESULT_GENERATOROVERFLOW
    cdef int FID_RESULT_SYSTIMEISINPAST
    cdef int FID_RESULT_WRONGSLICESIZE
    cdef int FID_RESULT_BASE64DECODEERROR
    cdef int FID_RESULT_BUFFERWRONGSIZE

    int32_t flowerid_new(FID *self, uint64_t timestamp, uint64_t sequence, uint64_t generator)
    int32_t flowerid_to_bytes(FID self, uint8_t *buffer, size_t buffer_size)
    int32_t flowerid_from_bytes(FID *self, const uint8_t *buffer, size_t buffer_size)
    int32_t flowerid_to_string(FID self, char *buffer, size_t buffer_size)
    int32_t flowerid_from_string(FID *self, const char *buffer)
    uint64_t flowerid_get_timestamp(FID self)
    uint64_t flowerid_get_sequence(FID self)
    uint64_t flowerid_get_generator(FID self)

    int32_t flowerid_generator_new(FID_GENERATOR *self, uint64_t generator, int32_t wait_sequence)
    int32_t flowerid_generator_new_ex(FID_GENERATOR *self, uint64_t generator, int64_t timestamp_offset, uint64_t timestamp_last, uint64_t sequence, int32_t wait_sequence, int32_t timestamp_in_seconds)
    int32_t flowerid_generator_next(FID_GENERATOR self, FID *dst)
    int32_t flowerid_generator_release(FID_GENERATOR self)

	
class Error(Exception):
    def __init__(self, code):
        self.__code = code

    def __str__(self):
        cdef int code = self.__code
        if code == FID_RESULT_OK:
            return "no error"
        elif code > FID_RESULT_OK:
            return "no error but failed `{}`".format(code)
        elif code == FID_RESULT_INVALIDARGUMENT:
            return "invalid argument error"
        elif code == FID_RESULT_TIMESTAMPOVERFLOW:
            return "timestamp overflow error"
        elif code == FID_RESULT_SEQUENCEOVERFLOW:
            return "sequence overflow error"
        elif code == FID_RESULT_GENERATOROVERFLOW:
            return "generator overflow error"
        elif code == FID_RESULT_SYSTIMEISINPAST:
            return "system time is in past error"
        elif code == FID_RESULT_WRONGSLICESIZE:
            return "wrong slice size error"
        elif code == FID_RESULT_BASE64DECODEERROR:
            return "base64 decode error"
        elif code == FID_RESULT_BUFFERWRONGSIZE:
            return "wrong buffer size error"
        return "unknown error"

    @property
    def error_code(self):
        self.__code

		
cdef class FId:
    cdef FID __fid
    def __init__(self, timestamp, sequance=None, generator=None):
        if isinstance(timestamp, int):
            if sequance is None and generator is None:
                self.__fid = timestamp
            else:
                assert(isinstance(sequance, int))
                assert(isinstance(generator, int))
                self.__fid = FId.__from_components(timestamp, sequance, generator)
        elif isinstance(timestamp, str):
            self.__fid = FId.__from_string(timestamp)
        elif isinstance(timestamp, bytes) or isinstance(timestamp, bytearray):
            self.__fid = FId.__from_bytes(timestamp)
        else:
            raise Error(FID_RESULT_INVALIDARGUMENT)

    @staticmethod
    def __from_components(uint64_t timestamp, uint64_t sequance, uint64_t generator):
        cdef FID res
        cdef int32_t err = flowerid_new(&res, timestamp, sequance, generator)
        if err != 0:
            raise Error(err)
        return res

    @staticmethod
    def __from_string(str origin):
        cdef FID res
        origin_bytes = origin.encode()
        cdef int32_t err = flowerid_from_string(&res, origin_bytes)
        if err != 0:
            raise Error(err)
        return res

    @staticmethod
    def __from_bytes(bytes origin):
        cdef FID res
        cdef int32_t err = flowerid_from_bytes(&res, origin, len(origin))
        if err != 0:
            raise Error(err)
        return res

    @property
    def timestamp(self):
        return flowerid_get_timestamp(self.__fid)

    @property
    def sequence(self):
        return flowerid_get_sequence(self.__fid)

    @property
    def generator(self):
        return flowerid_get_generator(self.__fid)

    def __str__(self):
        cdef char[12] res
        cdef int32_t err = flowerid_to_string(self.__fid, res, sizeof(res))
        if err != 11:
            raise Error(err)
        return bytes(res).decode()

    def __int__(self):
        return self.__fid

    def __lt__(self, FId other):
        return self.__fid < other.__fid

    def __le__(self, FId other):
        return self.__fid <= other.__fid

    def __gt__(self, FId other):
        return self.__fid > other.__fid

    def __ge__(self, FId other):
        return self.__fid >= other.__fid

    def __eq__(self, FId other):
        return self.__fid == other.__fid

    def __ne__(self, FId other):
        return self.__fid != other.__fid

    def serialize(self):
        cdef array.array res = array.array('B', [0] * 8)
        cdef int32_t err = flowerid_to_bytes(self.__fid, res.data.as_uchars, len(res))
        if err != 8:
            raise Error(err)
        return bytes(res)

		
cdef class FidGeneratorBuilder:
    cdef uint64_t __generator
    cdef int64_t  __timestamp_offset
    cdef uint64_t __timestamp_last
    cdef uint64_t __sequence
    cdef int32_t  __wait_sequence
    cdef int32_t  __timestamp_in_seconds

    def __init__(self, uint64_t generator):
        self.__generator = generator
        self.__timestamp_offset = -1483228800
        self.__timestamp_last = 0
        self.__sequence = 0
        self.__wait_sequence = 1
        self.__timestamp_in_seconds = 0

    def timestamp_offset(self, int64_t timestamp_offset):
        self.__timestamp_offset = timestamp_offset
        return self

    def timestamp_last(self, uint64_t timestamp_last):
        self.__timestamp_last = timestamp_last
        return self

    def sequence(self, uint64_t sequence):
        self.__sequence = sequence
        return self

    def wait_sequence(self):
        self.__wait_sequence = 1
        return self

    def not_wait_sequence(self):
        self.__wait_sequence = 0
        return self

    def timestamp_in_seconds(self):
        self.__timestamp_in_seconds = 1
        return self

    def timestamp_in_millisecond(self):
        self.__timestamp_in_seconds = 0
        return self

    def get_generator(self):
        return self.__generator

    def get_timestamp_offset(self):
        return self.__timestamp_offset

    def get_timestamp_last(self):
        return self.__timestamp_last

    def get_sequence(self):
        return self.__sequence

    def get_wait_sequence(self):
        return self.__wait_sequence

    def get_timestamp_in_seconds(self):
        return self.__timestamp_in_seconds

    def build(self):
        return FIdGenerator(self)


cdef class FIdGenerator:
    cdef FID_GENERATOR __gen
    def __init__(self, FidGeneratorBuilder cfg):
        cdef FID_GENERATOR gen
        cdef int32_t err = flowerid_generator_new_ex(&gen, cfg.__generator, cfg.__timestamp_offset, cfg.__timestamp_last, cfg.__sequence, cfg.__wait_sequence, cfg.__timestamp_in_seconds)
        if err != 0:
            raise Error(err)
        self.__gen = gen

    def next(self):
        cdef FID res
        cdef int32_t err = flowerid_generator_next(self.__gen, &res);
        if err != 0:
            raise Error(err)
        return FId(res)

    def __dealloc__(self):
        flowerid_generator_release(self.__gen)
