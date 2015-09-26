//! Library that allows to read and write primitive types from byte array with specified endianess.
//!
//! Implements ``` read_as ``` and ``` write_as ``` for: ``` std::io::Write ``` and ``` std::io::Read ```
//!
//! Supported types: ```u8``` ```u16``` ```u32``` ```u64```
//! ```i8``` ```i16``` ```i32``` ```i64``` ```f32``` ```f64```
//!
//! # Examples
//! ```
//! // Read
//! use endianrw::{BigEndian, LittleEndian, EndianReadExt};
//!
//! let data: Vec<u8> = vec![0x01, 0x23, 0x45, 0x67];
//!
//! assert_eq!(0x01234567, (&data[..]).read_as::<BigEndian, u32>().unwrap());
//! assert_eq!(0x67452301, (&data[..]).read_as::<LittleEndian, u32>().unwrap());
//! ```
//! ```
//! // Write
//! use endianrw::{BigEndian, LittleEndian, EndianWriteExt};
//!
//! let val = 0x01234567;
//! let mut data: Vec<u8> = vec![0; 4];
//! (&mut data[..]).write_as::<BigEndian, u32>(val).unwrap();
//! assert_eq!(&[0x01, 0x23, 0x45, 0x67], &data[..]);
//!
//! (&mut data[..]).write_as::<LittleEndian, u32>(val).unwrap();
//! assert_eq!(&[0x67, 0x45, 0x23, 0x01], &data[..]);
//! ```

use std::io;
use std::mem::transmute;

trait AsSlice<T>: AsRef<[T]> + AsMut<[T]> {}
impl<T, V: AsRef<[T]> + AsMut<[T]>> AsSlice<T> for V {}

/// Tranform primitive types to and from buffer.
pub trait ByteTransform<T> {
    /// Describes large enough buffer to store T
    type Buffer: AsSlice<u8>;

    /// Read T from buffer
    fn from_bytes(buf: Self::Buffer) -> T;

    /// Convert T to buffer
    fn to_bytes(val: T) -> Self::Buffer;

    /// Create large enough buffer to store T
    fn buffer() -> Self::Buffer;
}

/// Big endian byte order
pub enum BigEndian {}

/// Little endian byte order
pub enum LittleEndian {}

/// Network byte order
pub type NetworkByteOrder = BigEndian;
#[cfg(target_endian = "little")]
pub type NativeByteOrder = LittleEndian;
#[cfg(target_endian = "big")]
pub type NativeByteOrder = BigEndian;

macro_rules! impl_bytetransform {
    ($byteorder:ident, $convertfn:ident) => {
        impl_bytetransform!($byteorder, u8, 1, $convertfn);
        impl_bytetransform!($byteorder, u16, 2, $convertfn);
        impl_bytetransform!($byteorder, u32, 4, $convertfn);
        impl_bytetransform!($byteorder, u64, 8, $convertfn);
        impl_bytetransform!($byteorder, i8, 1, $convertfn);
        impl_bytetransform!($byteorder, i16, 2, $convertfn);
        impl_bytetransform!($byteorder, i32, 4, $convertfn);
        impl_bytetransform!($byteorder, i64, 8, $convertfn);
        impl_bytetransform!($byteorder, f32, 4, $convertfn, u32);
        impl_bytetransform!($byteorder, f64, 8, $convertfn, u64);
    };

    // Integer ByteTransform
    ($byteorder:ident, $typename:ident, $typesize:expr, $convertfn:ident) => {
        impl ByteTransform<$typename> for $byteorder {
            type Buffer = [u8; $typesize];

            #[inline]
            fn from_bytes(buf: Self::Buffer) -> $typename {
                unsafe { transmute::<_, $typename>(buf) }.$convertfn()
            }

            #[inline]
            fn to_bytes(val: $typename) -> Self::Buffer {
                unsafe { transmute(val.$convertfn()) }
            }

            #[inline]
            fn buffer() -> Self::Buffer {
                [0; $typesize]
            }
        }
    };

    // Floating point ByteTransform (requires additional transmute)
    ($byteorder:ident, $typename:ident, $typesize:expr, $convertfn:ident, $convertas:ident) => {
        impl ByteTransform<$typename> for $byteorder {
            type Buffer = [u8; $typesize];

            #[inline]
            fn from_bytes(buf: Self::Buffer) -> $typename {
                unsafe { transmute(transmute::<_, $convertas>(buf).$convertfn()) }
            }

            #[inline]
            fn to_bytes(val: $typename) -> Self::Buffer {
                unsafe { transmute(transmute::<_, $convertas>(val).$convertfn()) }
            }

            #[inline]
            fn buffer() -> Self::Buffer {
                [0; $typesize]
            }
        }

    }
}

impl_bytetransform!(LittleEndian, to_le);
impl_bytetransform!(BigEndian, to_be);

/// Extension trait that allows to read endian specified primitive types from it
pub trait EndianReadExt {
    fn read_as<B: ByteTransform<T>, T>(&mut self) -> io::Result<T>;
}

/// Extension trait that allows to write endian specified primitive types to it
pub trait EndianWriteExt {
    fn write_as<B: ByteTransform<T>, T>(&mut self, val: T) -> io::Result<()>;
}

impl<R: io::Read> EndianReadExt for R {
    #[inline]
    fn read_as<B: ByteTransform<T>, T>(&mut self) -> io::Result<T> {
        let mut buf = B::buffer();
        let read_len = try!(self.read(buf.as_mut()));
        if read_len != buf.as_ref().len() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "could not read all bytes"
            ))
        }
        Ok(B::from_bytes(buf))
    }
}

impl<W: io::Write> EndianWriteExt for W {
    #[inline]
    fn write_as<B: ByteTransform<T>, T>(&mut self, val: T) -> io::Result<()> {
        let buf = B::to_bytes(val);
        self.write_all(buf.as_ref())
    }
}

#[cfg(test)]
mod test {
    use super::{BigEndian, LittleEndian, EndianReadExt, EndianWriteExt};

    #[test]
    fn test_all() {
        let expected = [0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88];

        macro_rules! run_test {
            ($typename:ident, $typesize:expr, $big:expr, $little:expr) => {
                run_test!($typename, $typesize, BigEndian, $big, true);
                run_test!($typename, $typesize, LittleEndian, $little, true);
            };

            ($typename:ident, $typesize:expr, $order:ident, $value:expr, $inner:expr) => {
                {
                    // Test reading
                    let val = (&expected[0..$typesize]).read_as::<$order, $typename>().unwrap();
                    assert_eq!($value, val);
                    // Test writing
                    let mut buf: Vec<u8> = vec![0; $typesize];
                    (&mut buf[0..$typesize]).write_as::<$order, $typename>($value).unwrap();
                    assert_eq!(&expected[..$typesize], &buf[..]);
                    // Read from too few bytes
                    (&expected[0..$typesize - 1]).read_as::<$order, $typename>().unwrap_err();
                    // Write to too few bytes
                    let mut buf: Vec<u8> = vec![0; $typesize];
                    (&mut buf[0..$typesize - 1]).write_as::<$order, $typename>($value).unwrap_err();
                }
            };
        }

        // Tests are checked against array [0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88]
        // run_test!(TYPE, VALUE_IF_ARRAY_BIG_ENDIAN, VALUE_IF_ARRAY_LITTLE_ENDIAN);
        // for types that are less than 8 bytes long first N bytes are used

        run_test!(u8, 1,  17, 17);
        run_test!(u16, 2, 4386, 8721);
        run_test!(u32, 4, 287454020, 1144201745);
        run_test!(u64, 8, 1234605616436508552, 9833440827789222417);

        run_test!(i8, 1, 17, 17);
        run_test!(i16, 2, 4386, 8721);
        run_test!(i32, 4, 287454020, 1144201745);
        run_test!(i64, 8, 1234605616436508552, -8613303245920329199);

        run_test!(f32, 4, 1.2795344e-28, 7.165323e2);
        run_test!(f64, 8, 3.841412024471731e-226, -7.086876636573014e-268);
    }
}
