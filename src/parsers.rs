pub fn parse_sha1(s: &str) -> Result<String, String> {
    if s.len() != 40 {
        return Err("SHA1 must be 40 characters".into());
    }

    if !s.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("SHA1 must be hex".into());
    }

    Ok(s.to_string())
}
