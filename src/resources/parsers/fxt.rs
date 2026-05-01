use crate::resources::types::text::TextBundle;
use crate::resources::Result;
use std::collections::HashMap;

pub fn parse_fxt(data: &[u8]) -> Result<TextBundle> {
    let decrypted = decrypt_fxt(data);
    
    let mut entries = HashMap::new();
    let mut current_pos = 0;
    
    while current_pos < decrypted.len() {
        // Find end of entry (null terminator)
        let end_pos = decrypted[current_pos..]
            .iter()
            .position(|&b| b == 0)
            .map(|p| current_pos + p)
            .unwrap_or(decrypted.len());
            
        let entry_data = &decrypted[current_pos..end_pos];
        if !entry_data.is_empty() {
            if let Some((key, value)) = parse_entry(entry_data) {
                entries.insert(key, value);
            }
        }
        
        current_pos = end_pos + 1;
    }
    
    Ok(TextBundle { entries })
}

fn decrypt_fxt(data: &[u8]) -> Vec<u8> {
    let mut decrypted = data.to_vec();
    
    // Decrypt first 8 bytes
    for i in 0..std::cmp::min(8, decrypted.len()) {
        let shift_val = (99u128 << i) as u8;
        decrypted[i] = decrypted[i].wrapping_sub(shift_val).wrapping_sub(1);
    }
    
    // Decrypt remaining bytes
    for i in 8..decrypted.len() {
        decrypted[i] = decrypted[i].wrapping_sub(1);
    }
    
    decrypted
}

fn parse_entry(data: &[u8]) -> Option<(String, String)> {
    let s = String::from_utf8_lossy(data);
    if s.starts_with('[') {
        if let Some(end_bracket) = s.find(']') {
            let key = s[1..end_bracket].to_string();
            let value = s[end_bracket + 1..].to_string();
            return Some((key, value));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decrypt_fxt() {
        // [1001] starts at 0xBF 0xF8 ...
        let data = vec![0xbf, 0xf8, 0xbd, 0x49, 0x62, 0xbe, 0x02, 0xef];
        let decrypted = decrypt_fxt(&data);
        assert_eq!(&decrypted[0..6], b"[1001]");
    }

    #[test]
    fn test_parse_fxt() {
        let mut data = vec![0xbf, 0xf8, 0xbd, 0x49, 0x62, 0xbe, 0x02, 0xef]; // [1001]An
        data.extend_from_slice(b"swer the phone");
        for i in 8..data.len() {
            data[i] = data[i].wrapping_add(1);
        }
        data.push(1); // null terminator (0 + 1)
        
        let bundle = parse_fxt(&data).unwrap();
        assert_eq!(bundle.get("1001").unwrap(), "Answer the phone");
    }
}
