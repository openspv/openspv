use crate::blake3::{blake3_hash, double_blake3_hash};
use crate::block_header::BlockHeader;
use crate::buffer_reader::BufferReader;
use crate::buffer_writer::BufferWriter;
use crate::tx::Tx;
use crate::var_int::VarInt;

pub struct Block {
    pub header: BlockHeader,
    pub txs: Vec<Tx>,
}

impl Block {
    pub fn new(header: BlockHeader, txs: Vec<Tx>) -> Self {
        Self { header, txs }
    }

    pub fn from_buffer_reader(br: &mut BufferReader) -> Result<Self, Box<dyn std::error::Error>> {
        let header = BlockHeader::from_buffer_reader(br);
        let tx_count_varint = VarInt::from_buffer_reader(br);
        if !tx_count_varint.is_minimal() {
            return Err("non-minimally encoded varint".into());
        }
        let tx_count = tx_count_varint.to_u64() as usize;
        let mut txs = vec![];
        for _ in 0..tx_count {
            let tx = Tx::from_buffer_reader(br).map_err(|_| "unable to parse transactions")?;
            txs.push(tx);
        }
        Ok(Self { header, txs })
    }

    pub fn to_buffer_writer(&self) -> BufferWriter {
        let mut bw = BufferWriter::new();
        bw.write_u8_vec(self.header.to_u8_vec());
        bw.write_u8_vec(VarInt::from_u64_new(self.txs.len() as u64).to_u8_vec());
        for tx in &self.txs {
            bw.write_u8_vec(tx.to_buffer_writer().to_u8_vec());
        }
        bw
    }

    pub fn to_u8_vec(&self) -> Vec<u8> {
        self.to_buffer_writer().to_u8_vec()
    }

    pub fn from_u8_vec(buf: Vec<u8>) -> Result<Self, Box<dyn std::error::Error>> {
        let mut br = BufferReader::new(buf);
        Self::from_buffer_reader(&mut br)
    }

    pub fn hash(&self) -> Vec<u8> {
        blake3_hash(&self.to_u8_vec()).to_vec()
    }

    pub fn id(&self) -> Vec<u8> {
        double_blake3_hash(&self.to_u8_vec()).to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_buffer_writer() {
        let header = BlockHeader::new(1, vec![0; 32], vec![0; 32], 1, vec![0; 32], vec![0; 32], 1);
        let tx = Tx::new(1, vec![], vec![], 1);
        let block = Block::new(header, vec![tx]);
        let bw = block.to_buffer_writer();
        assert_eq!(bw.to_u8_vec().len() > 0, true);
    }

    #[test]
    fn test_to_u8_vec_and_from_u8_vec() {
        let header = BlockHeader::new(1, vec![0; 32], vec![0; 32], 1, vec![0; 32], vec![0; 32], 1);
        let tx = Tx::new(1, vec![], vec![], 1);
        let block1 = Block::new(header, vec![tx]);
        let buf = block1.to_u8_vec();
        let block2 = Block::from_u8_vec(buf).unwrap();
        assert_eq!(block1.header.version, block2.header.version);
        assert_eq!(block1.txs[0].version, block2.txs[0].version);
    }

    #[test]
    fn test_from_buffer_reader() {
        let header = BlockHeader::new(1, vec![0; 32], vec![0; 32], 1, vec![0; 32], vec![0; 32], 1);
        let tx = Tx::new(1, vec![], vec![], 1);
        let block1 = Block::new(header, vec![tx]);
        let buf = block1.to_u8_vec();
        let mut br = BufferReader::new(buf);
        let block2 = Block::from_buffer_reader(&mut br).unwrap();
        assert_eq!(block1.header.version, block2.header.version);
        assert_eq!(block1.txs[0].version, block2.txs[0].version);
    }

    #[test]
    fn test_hash() {
        let header = BlockHeader::new(1, vec![0; 32], vec![0; 32], 0, vec![0; 32], vec![0; 32], 0);
        let tx = Tx::new(1, vec![], vec![], 0);
        let block = Block::new(header, vec![tx]);
        let hash_buf = block.hash();
        let hash_hex = hex::encode(hash_buf);
        assert_eq!(
            hash_hex,
            "52e5f4747e31dd27920febfc098c18aa68aa16b316809682dc5194cd3b247c17"
        );
    }

    #[test]
    fn test_id() {
        let header = BlockHeader::new(1, vec![0; 32], vec![0; 32], 0, vec![0; 32], vec![0; 32], 0);
        let tx = Tx::new(1, vec![], vec![], 0);
        let block = Block::new(header, vec![tx]);
        let id_buf = block.id();
        let id_hex = hex::encode(id_buf);
        assert_eq!(
            id_hex,
            "082081dcb7b17538d3e40ceb876fe70e8e2c9d29f68ed7c55d6ce3a3e23b972f"
        );
    }

    #[test]
    fn test_is_genesis() {
        let header = BlockHeader::new(1, vec![0; 32], vec![0; 32], 0, vec![0; 32], vec![0; 32], 0);
        let tx = Tx::new(1, vec![], vec![], 0);
        let block = Block::new(header, vec![tx]);
        assert_eq!(block.header.is_genesis(), true);
    }
}