## Flower ID structure

Length: 8 octets

| Sign     | Timestamp         | Sequence | Generator    |
| :------: | :---------------: | :-----:  | :----------: |
| 1bits    | 42bits            | 11bits   | 10bi         |
| always 0 | msec from u-epoch | sequence | generator id |

* Sign:      always 0
* Timestamp: num of millisecond(or second) since (01.01.2017 00:00:00 UTC+0)
             offset from unix -1483228800 * 10^3
             Limits: 4398046511103 msec (~139 years)
             End of life: 05.15.2156 07:35 UTC+0
* Sequence:  counter to evade collision, reset to 0 after timestamp incremented
             Max value: 2047
* Generator: generator unique id
             Max value: 1023

## Flower ID binary representation

Length: 8 octets
Encoding: big-endian

## Flower ID string representation

Encoding: url-safe base64
* Base64: [RFC 3548 §4](https://tools.ietf.org/html/rfc3548#section-4)
          optionaly - without last pad symbol(=)
          19-20 symbols
