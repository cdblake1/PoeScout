use base64::Engine;
use flate2::read::ZlibDecoder;
use serde::{Deserialize, Serialize};
use std::io::Read;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildSummary {
    pub class_name: String,
    pub ascendancy: String,
    pub level: i32,
    pub main_skill: Option<String>,
    pub total_stats: BuildStats,
    pub xml_raw: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BuildStats {
    pub life: Option<String>,
    pub energy_shield: Option<String>,
    pub mana: Option<String>,
    pub str_val: Option<String>,
    pub dex_val: Option<String>,
    pub int_val: Option<String>,
}

/// Decodes a PoB build code (base64 → zlib inflate → XML).
/// Accepts raw codes or pobb.in/pastebin URLs (extracts the code portion).
pub fn decode_build_code(input: &str) -> Result<BuildSummary, String> {
    let code = extract_code(input);
    let xml = decode_to_xml(code)?;
    parse_xml_summary(&xml)
}

/// Extract the code portion from a URL or raw string.
fn extract_code(input: &str) -> &str {
    let trimmed = input.trim();

    // pobb.in URLs: https://pobb.in/XXXXX — the code is the path segment
    if let Some(rest) = trimmed.strip_prefix("https://pobb.in/") {
        return rest.split(['?', '#', '/']).next().unwrap_or(rest);
    }

    // pastebin: https://pastebin.com/XXXXX
    if let Some(rest) = trimmed.strip_prefix("https://pastebin.com/") {
        let slug = rest.strip_prefix("raw/").unwrap_or(rest);
        return slug.split(['?', '#', '/']).next().unwrap_or(slug);
    }

    trimmed
}

/// base64 decode → zlib inflate → UTF-8 XML string
fn decode_to_xml(code: &str) -> Result<String, String> {
    // PoB uses URL-safe base64 with no padding
    let engine = base64::engine::general_purpose::URL_SAFE_NO_PAD;
    let bytes = engine
        .decode(code)
        .or_else(|_| {
            // Fallback: try standard base64
            base64::engine::general_purpose::STANDARD.decode(code)
        })
        .map_err(|e| format!("Base64 decode failed: {}", e))?;

    let mut decoder = ZlibDecoder::new(&bytes[..]);
    let mut xml = String::new();
    decoder
        .read_to_string(&mut xml)
        .map_err(|e| format!("Zlib inflate failed: {}", e))?;

    Ok(xml)
}

/// Parse the PoB XML to extract a build summary.
fn parse_xml_summary(xml: &str) -> Result<BuildSummary, String> {
    let mut class_name = String::new();
    let mut ascendancy = String::new();
    let mut level: i32 = 1;
    let mut main_skill: Option<String> = None;
    let mut stats = BuildStats::default();

    let mut reader = quick_xml::Reader::from_str(xml);
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(quick_xml::events::Event::Empty(ref e)) | Ok(quick_xml::events::Event::Start(ref e)) => {
                match e.name().as_ref() {
                    b"Build" => {
                        for attr in e.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"className" => {
                                    class_name = String::from_utf8_lossy(&attr.value).to_string();
                                }
                                b"ascendClassName" => {
                                    ascendancy = String::from_utf8_lossy(&attr.value).to_string();
                                }
                                b"level" => {
                                    level = String::from_utf8_lossy(&attr.value)
                                        .parse()
                                        .unwrap_or(1);
                                }
                                b"mainSocketGroup" => {
                                    // We'll resolve the name separately
                                }
                                _ => {}
                            }
                        }
                    }
                    b"PlayerStat" => {
                        let mut stat_name = String::new();
                        let mut stat_val = String::new();
                        for attr in e.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"stat" => {
                                    stat_name = String::from_utf8_lossy(&attr.value).to_string();
                                }
                                b"value" => {
                                    stat_val = String::from_utf8_lossy(&attr.value).to_string();
                                }
                                _ => {}
                            }
                        }
                        match stat_name.as_str() {
                            "Life" => stats.life = Some(stat_val),
                            "EnergyShield" => stats.energy_shield = Some(stat_val),
                            "Mana" => stats.mana = Some(stat_val),
                            "Str" => stats.str_val = Some(stat_val),
                            "Dex" => stats.dex_val = Some(stat_val),
                            "Int" => stats.int_val = Some(stat_val),
                            _ => {}
                        }
                    }
                    b"Skill" => {
                        // Grab first skill's label as main skill if we don't have one
                        if main_skill.is_none() {
                            for attr in e.attributes().flatten() {
                                if attr.key.as_ref() == b"label" {
                                    let label = String::from_utf8_lossy(&attr.value).to_string();
                                    if !label.is_empty() {
                                        main_skill = Some(label);
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(quick_xml::events::Event::Eof) => break,
            Err(e) => return Err(format!("XML parse error: {}", e)),
            _ => {}
        }
        buf.clear();
    }

    if class_name.is_empty() {
        return Err("Could not find <Build> element in XML".to_string());
    }

    Ok(BuildSummary {
        class_name,
        ascendancy,
        level,
        main_skill,
        total_stats: stats,
        xml_raw: xml.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_code_raw() {
        assert_eq!(extract_code("  abc123  "), "abc123");
    }

    #[test]
    fn test_extract_code_pobb() {
        assert_eq!(extract_code("https://pobb.in/XyZ123"), "XyZ123");
    }

    #[test]
    fn test_extract_code_pastebin() {
        assert_eq!(extract_code("https://pastebin.com/AbC456"), "AbC456");
        assert_eq!(extract_code("https://pastebin.com/raw/AbC456"), "AbC456");
    }
}
