use crate::resources::{Result, Error};
use std::io::{Read, Cursor};

#[derive(Debug, Clone, PartialEq)]
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
        let offset = read_u32(&mut cursor)?;
        let size = read_u32(&mut cursor)?;
        let frequency = read_u32(&mut cursor)?;
        indices.push(SoundIndex { offset, size, frequency });
    }
    
    Ok(indices)
}

fn read_u32(r: &mut Cursor<&[u8]>) -> Result<u32> {
    let mut buf = [0u8; 4];
    r.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_sdt() {
        let mut data = Vec::new();
        // Record 1
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
