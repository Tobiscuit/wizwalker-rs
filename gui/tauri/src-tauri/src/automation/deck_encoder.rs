//! Deck encoder/decoder — encode decks to shareable tokens and back.
//!
//! Faithfully ported from `deimos-reference/src/deck_encoder.py`.
//!
//! Tokens are zlib-compressed, base64-encoded strings representing a deck's
//! contents (normal cards, treasure cards, item cards) with their quantities.
#![allow(dead_code, unused_imports)]

use std::collections::HashMap;
use std::io::{Read, Write};

const DELIMITER1: &str = "\\+";
const DELIMITER2: &str = "\\,";
const DELIMITER3: &str = "\\;";

/// A deck with three sections: normal, tc (treasure cards), item.
#[derive(Debug, Clone, Default)]
pub struct Deck {
    pub normal: HashMap<String, u32>,
    pub tc: HashMap<String, u32>,
    pub item: HashMap<String, u32>,
}

/// Encode a deck into a shareable token string.
///
/// Flow: serialize → zlib compress → base64 encode.
///
/// Python: `DeckEncoderDecoder.encode()` — deck_encoder.py:34
pub fn encode_deck(deck: &Deck) -> Result<String, String> {
    let sections = [
        serialize_section(&deck.normal),
        serialize_section(&deck.tc),
        serialize_section(&deck.item),
    ];
    let deck_str = sections.join(DELIMITER3);

    // Zlib compress
    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    encoder
        .write_all(deck_str.as_bytes())
        .map_err(|e| format!("Compression failed: {e}"))?;
    let compressed = encoder
        .finish()
        .map_err(|e| format!("Compression finish failed: {e}"))?;

    // Base64 encode
    use base64::Engine as _;
    Ok(base64::engine::general_purpose::STANDARD.encode(&compressed))
}

/// Decode a token string back into a Deck.
///
/// Flow: base64 decode → zlib decompress → deserialize.
///
/// Python: `DeckEncoderDecoder.decode()` — deck_encoder.py:57
pub fn decode_deck(token: &str) -> Result<Deck, String> {
    use base64::Engine as _;

    // Base64 decode
    let compressed = base64::engine::general_purpose::STANDARD
        .decode(token.as_bytes())
        .map_err(|e| format!("Base64 decode failed: {e}"))?;

    // Zlib decompress
    let mut decoder = flate2::read::ZlibDecoder::new(&compressed[..]);
    let mut decompressed = String::new();
    decoder
        .read_to_string(&mut decompressed)
        .map_err(|e| format!("Decompression failed: {e}"))?;

    // Deserialize sections
    let sections: Vec<&str> = decompressed.split(DELIMITER3).collect();
    if sections.len() != 3 {
        return Err(format!(
            "Invalid token format: expected 3 sections, got {}",
            sections.len()
        ));
    }

    Ok(Deck {
        normal: deserialize_section(sections[0]),
        tc: deserialize_section(sections[1]),
        item: deserialize_section(sections[2]),
    })
}

/// Serialize a section (card name → count map) into a compact string.
///
/// Python: `_serialize_section(section)` — deck_encoder.py:18
fn serialize_section(section: &HashMap<String, u32>) -> String {
    if section.is_empty() {
        return "N".to_string();
    }
    section
        .iter()
        .map(|(key, value)| format!("{}{}{}", key, DELIMITER1, value))
        .collect::<Vec<_>>()
        .join(DELIMITER2)
}

/// Deserialize a section string back into a card name → count map.
///
/// Python: `_deserialize_section(section)` — deck_encoder.py:26
fn deserialize_section(section: &str) -> HashMap<String, u32> {
    if section == "N" {
        return HashMap::new();
    }
    section
        .split(DELIMITER2)
        .filter_map(|item| {
            let parts: Vec<&str> = item.split(DELIMITER1).collect();
            if parts.len() == 2 {
                let count: u32 = parts[1].parse().unwrap_or(0);
                Some((parts[0].to_string(), count))
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip() {
        let mut deck = Deck::default();
        deck.normal.insert("Colossal".to_string(), 5);
        deck.normal.insert("Giant".to_string(), 7);
        deck.tc.insert("Frost Beetle TC".to_string(), 1);
        deck.item.insert("Moon Shield".to_string(), 1);

        let token = encode_deck(&deck).expect("encode should work");
        let decoded = decode_deck(&token).expect("decode should work");

        assert_eq!(decoded.normal.get("Colossal"), Some(&5));
        assert_eq!(decoded.normal.get("Giant"), Some(&7));
        assert_eq!(decoded.tc.get("Frost Beetle TC"), Some(&1));
        assert_eq!(decoded.item.get("Moon Shield"), Some(&1));
    }
}
