use crate::resources::{Result, Error};
use std::io::Cursor;
use binrw::{BinRead, BinReaderExt};

#[derive(Debug, Clone, PartialEq, BinRead)]
#[br(little)]
pub struct SoundIndex {
    pub offset: u32,
    pub size: u32,
    pub frequency: u32,
}

pub fn parse_sdt(data: &[u8]) -> Result<Vec<SoundIndex>> {
    if data.len() % 12 != 0 {
        return Err(Error::Parse(format!("SDT file size {} is not a multiple of 12", data.len())));
    }
    
    let mut cursor = Cursor::new(data);
    let num_records = data.len() / 12;
    let mut indices = Vec::with_capacity(num_records);
    
    for _ in 0..num_records {
        indices.push(cursor.read_le()?);
    }
    
    Ok(indices)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_sdt() {
        let mut data = Vec::new();
        data.extend_from_slice(&100u32.to_le_bytes());
        data.extend_from_slice(&200u32.to_le_bytes());
        data.extend_from_slice(&22050u32.to_le_bytes());
        
        let indices = parse_sdt(&data).unwrap();
        assert_eq!(indices.len(), 1);
        assert_eq!(indices[0].offset, 100);
        assert_eq!(indices[0].size, 200);
        assert_eq!(indices[0].frequency, 22050);
    }
}
