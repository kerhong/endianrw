# endianrw

Rust library for reading and writing numbers with specific endianness.

Heavily influenced by [byteorder](https://github.com/BurntSushi/byteorder), but with more generic API.

[![Build Status](https://img.shields.io/travis/kerhong/endianrw.svg)](https://travis-ci.org/kerhong/endianrw)
[![Crates.io](https://img.shields.io/crates/v/endianrw.svg)](https://crates.io/crates/endianrw)

## License
MIT

## Documentation
[https://kerhong.github.io/endianrw](https://kerhong.github.io/endianrw)

## Examples
### Read
``` rust
use endianrw::{BigEndian, LittleEndian, EndianReadExt};

let data: Vec<u8> = vec![0x01, 0x23, 0x45, 0x67];

assert_eq!(0x01234567, (&data[..]).read_as::<BigEndian, u32>().unwrap());
assert_eq!(0x67452301, (&data[..]).read_as::<LittleEndian, u32>().unwrap());
```

### Write
``` rust
use endianrw::{BigEndian, LittleEndian, EndianWriteExt};

let val = 0x01234567;
let mut data: Vec<u8> = vec![0; 4];
(&mut data[..]).write_as::<BigEndian, u32>(val).unwrap();
assert_eq!(0x01, data[0]);

(&mut data[..]).write_as::<LittleEndian, u32>(val).unwrap();
assert_eq!(0x67, data[0]);
```
