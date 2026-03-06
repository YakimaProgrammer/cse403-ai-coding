use std::collections::HashMap;
use csv::ReaderBuilder;

#[allow(dead_code)]
pub fn parse_csv(text: &str) -> Result<(Vec<String>, Vec<HashMap<String, String>>), String> {
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(text.as_bytes());

    let headers = reader.headers()
        .map_err(|e| e.to_string())?
        .iter()
        .map(|s| s.to_string())
        .collect();

    let data = reader.deserialize()
        .filter_map(|r: Result<HashMap<String, String>, _>| r.ok())
        .collect();

    Ok((headers, data))
}
