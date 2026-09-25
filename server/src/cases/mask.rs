//! Masking helpers for API responses (DESIGN.md privacy rules).
//!
//! Masked values are the only identity a list or detail response carries;
//! the full snapshot requires the audited reveal flow.

/// `สม*****` — the first two characters, then a fixed mask. Shorter names
/// mask entirely.
pub fn mask_name(name: &str) -> String {
    let first: String = name.chars().take(2).collect();
    if first.is_empty() {
        return "*****".to_string();
    }
    format!("{first}*****")
}

/// `12****90` — the first and last two characters. Short HN patterns mask
/// entirely rather than revealing most of the value.
pub fn mask_hn(hn: &str) -> String {
    let chars: Vec<char> = hn.chars().collect();
    if chars.len() <= 4 {
        return "****".to_string();
    }
    let first: String = chars.iter().take(2).collect();
    let last: String = chars.iter().skip(chars.len() - 2).collect();
    format!("{first}****{last}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_keeps_two_characters_then_masks() {
        assert_eq!(mask_name("สมชาย ทดสอบ"), "สม*****");
        assert_eq!(mask_name("A"), "A*****");
    }

    #[test]
    fn empty_name_masks_entirely() {
        assert_eq!(mask_name(""), "*****");
    }

    #[test]
    fn hn_keeps_edges_and_masks_the_middle() {
        assert_eq!(mask_hn("123456"), "12****56");
    }

    #[test]
    fn short_hn_masks_entirely() {
        assert_eq!(mask_hn("1234"), "****");
        assert_eq!(mask_hn("12"), "****");
    }
}
