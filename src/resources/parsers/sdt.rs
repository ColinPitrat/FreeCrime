use crate::resources::{Result, Error};
use std::io::Cursor;
use binrw::{BinRead, BinReaderExt};

/// Index entry into a RAW sound file.
#[derive(Debug, Clone, PartialEq, BinRead)]
#[br(little)]
pub struct SoundIndex {
    /// Offset of the sample in the .RAW file.
    pub offset: u32,
    /// Size of the sample in bytes.
    pub size: u32,
    /// Sampling frequency (e.g. 11025 or 22050).
    pub frequency: u32,
}

/// Parses an SDT sound index file.
/// These files contain a list of sample offsets and frequencies for an associated .RAW file.
pub fn parse_sdt(data: &[u8]) -> Result<Vec<SoundIndex>> {
    if !data.len().is_multiple_of(12) {
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
