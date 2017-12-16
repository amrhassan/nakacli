
pub fn unsigned_int(v: String) -> Result<(), String> {
    match v.parse::<u64>() {
        Ok(n) if n > 0 => Ok(()),
        _ => Err("Value should be a positive integer".to_string())
    }
}
