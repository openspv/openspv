use crate::numbers::u256;
use byteorder::{BigEndian, WriteBytesExt};

// add Default
#[derive(Default)]
pub struct BufWriter {
    bufs: Vec<Vec<u8>>,
}

impl BufWriter {
    pub fn new() -> BufWriter {
        BufWriter { bufs: Vec::new() }
    }

    pub fn with_buffers(buffers: Vec<Vec<u8>>) -> BufWriter {
        BufWriter { bufs: buffers }
    }

    pub fn get_length(&self) -> usize {
        self.bufs.iter().map(|buf| buf.len()).sum()
    }

    pub fn to_buf(&self) -> Vec<u8> {
        let mut result = Vec::new();
        for buf in &self.bufs {
            result.extend(buf);
        }
        result
    }

    pub fn write(&mut self, buf: Vec<u8>) -> &mut Self {
        self.bufs.push(buf);
        self
    }

    pub fn write_reverse(&mut self, buf: Vec<u8>) -> &mut Self {
        let mut buf2 = vec![0; buf.len()];
        for (i, &item) in buf.iter().rev().enumerate() {
            buf2[i] = item;
        }
        self.bufs.push(buf2);
        self
    }

    pub fn write_u8(&mut self, n: u8) -> &mut Self {
        self.bufs.push(vec![n]);
        self
    }

    pub fn write_u16_be(&mut self, n: u16) -> &mut Self {
        let mut buf = vec![0; 2];
        buf.as_mut_slice().write_u16::<BigEndian>(n).unwrap();
        self.bufs.push(buf);
        self
    }

    pub fn write_u32_be(&mut self, n: u32) -> &mut Self {
        let mut buf = vec![0; 4];
        buf.as_mut_slice().write_u32::<BigEndian>(n).unwrap();
        self.bufs.push(buf);
        self
    }

    pub fn write_u64_be(&mut self, n: u64) -> &mut Self {
        let mut buf = vec![0; 8];
        buf.as_mut_slice().write_u64::<BigEndian>(n).unwrap();
        self.bufs.push(buf);
        self
    }

    pub fn write_u128_be(&mut self, n: u128) -> &mut Self {
        let mut buf = vec![0; 16];
        buf.as_mut_slice().write_u128::<BigEndian>(n).unwrap();
        self.bufs.push(buf);
        self
    }

    pub fn write_u256_be(&mut self, n: u256) -> &mut Self {
        // digits() is little endian, so we need to reverse the order
        let digits = n.digits();
        let val1 = digits[0];
        let val2 = digits[1];
        let val3 = digits[2];
        let val4 = digits[3];
        self.write_u64_be(val4);
        self.write_u64_be(val3);
        self.write_u64_be(val2);
        self.write_u64_be(val1);
        self
    }

    pub fn var_int_buf(n: u64) -> Vec<u8> {
        let mut buf = vec![];
        if n < 253 {
            buf.push(n as u8);
        } else if n < 0x10000 {
            buf.push(253);
            buf.write_u16::<BigEndian>(n as u16).unwrap();
        } else if n < 0x100000000 {
            buf.push(254);
            buf.write_u32::<BigEndian>(n as u32).unwrap();
        } else {
            buf.push(255);
            buf.write_u64::<BigEndian>(n).unwrap();
        }
        buf
    }

    pub fn write_var_int(&mut self, n: u64) -> &mut Self {
        let buf = BufWriter::var_int_buf(n);
        self.write(buf);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_length() {
        let mut writer = BufWriter::new();
        assert_eq!(writer.get_length(), 0);

        writer.write_u8(1);
        assert_eq!(writer.get_length(), 1);

        writer.write(vec![2, 3, 4]);
        assert_eq!(writer.get_length(), 4);
    }

    #[test]
    fn test_to_buf() {
        let mut writer = BufWriter::new();
        writer.write(vec![1, 2, 3]);
        writer.write(vec![4, 5, 6]);

        let result = writer.to_buf();
        assert_eq!(result, vec![1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn test_write_iso_buf() {
        let mut writer = BufWriter::new();
        writer.write(vec![1, 2, 3]);

        assert_eq!(writer.bufs.len(), 1);
        assert_eq!(writer.bufs[0], vec![1, 2, 3]);
    }

    #[test]
    fn test_write_reverse() {
        let mut writer = BufWriter::new();
        writer.write_reverse(vec![1, 2, 3]);

        assert_eq!(writer.bufs.len(), 1);
        assert_eq!(writer.bufs[0], vec![3, 2, 1]);
    }

    #[test]
    fn test_write_u8() {
        let mut writer = BufWriter::new();
        writer.write_u8(1);

        assert_eq!(writer.bufs.len(), 1);
        assert_eq!(writer.bufs[0], vec![1]);
    }

    #[test]
    fn test_write_u16_be() {
        let mut writer = BufWriter::new();
        writer.write_u16_be(0x0102);

        assert_eq!(writer.bufs.len(), 1);
        assert_eq!(writer.bufs[0], vec![1, 2]); // 0x0102 in big-endian is [1, 2]
    }

    #[test]
    fn test_write_u32_be() {
        let mut writer = BufWriter::new();
        writer.write_u32_be(0x01020304);

        assert_eq!(writer.bufs.len(), 1);
        assert_eq!(writer.bufs[0], vec![1, 2, 3, 4]); // 0x01020304 in big-endian is [1, 2, 3, 4]
    }

    #[test]
    fn test_write_u64_be() {
        let mut writer = BufWriter::new();
        writer.write_u64_be(0x0102030405060708);

        assert_eq!(writer.bufs.len(), 1);
        assert_eq!(writer.bufs[0], vec![1, 2, 3, 4, 5, 6, 7, 8]); // 0x0102030405060708 in big-endian is [1, 2, 3, 4, 5, 6, 7, 8]
    }

    #[test]
    fn test_var_int_buf() {
        // Test case where n < 253
        let buf: Vec<u8> = vec![100];
        assert_eq!(BufWriter::var_int_buf(100), buf);

        // Test case where 253 <= n < 0x10000
        let mut buf = vec![];
        buf.push(253);
        buf.write_u16::<BigEndian>(0x0102).unwrap();
        assert_eq!(BufWriter::var_int_buf(0x0102), buf);

        // Test case where 0x10000 <= n < 0x100000000
        let mut buf = vec![];
        buf.push(254);
        buf.write_u32::<BigEndian>(0x01020304).unwrap();
        assert_eq!(BufWriter::var_int_buf(0x01020304), buf);

        // Test case where n >= 0x100000000
        let mut buf = vec![];
        buf.push(255);
        buf.write_u64::<BigEndian>(0x0102030405060708).unwrap();
        assert_eq!(BufWriter::var_int_buf(0x0102030405060708), buf);
    }

    #[test]
    fn test_write_var_int() {
        // Test case where n < 253
        let mut writer = BufWriter::new();
        writer.write_var_int(100);
        assert_eq!(writer.bufs.len(), 1);
        assert_eq!(writer.bufs[0], vec![100]);

        // Test case where 253 <= n < 0x10000
        let mut writer = BufWriter::new();
        writer.write_var_int(0x0102);
        assert_eq!(writer.bufs.len(), 1);
        let mut expected = vec![253];
        expected.write_u16::<BigEndian>(0x0102).unwrap();
        assert_eq!(writer.bufs[0], expected);

        // Test case where 0x10000 <= n < 0x100000000
        let mut writer = BufWriter::new();
        writer.write_var_int(0x01020304);
        assert_eq!(writer.bufs.len(), 1);
        let mut expected = vec![254];
        expected.write_u32::<BigEndian>(0x01020304).unwrap();
        assert_eq!(writer.bufs[0], expected);

        // Test case where n >= 0x100000000
        let mut writer = BufWriter::new();
        writer.write_var_int(0x0102030405060708);
        assert_eq!(writer.bufs.len(), 1);
        let mut expected = vec![255];
        expected.write_u64::<BigEndian>(0x0102030405060708).unwrap();
        assert_eq!(writer.bufs[0], expected);
    }
}
