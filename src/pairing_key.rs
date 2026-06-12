/// Returns a canonical pairing key derived from filenames in any of the known
/// naming conventions, or `None` when the filename follows none of them.
///
/// Bus-stop plates (6 components):
/// Side A: `路段名_站点名称_站点方位_站牌序号_尺寸_正|反`
/// Side B: `路段名_站点名称_站点方位_站架序号_尺寸_A|B`
/// 站牌序号 = (站架序号 - 1) * 2 + (A/正 -> 1, B/反 -> 2)
/// Key: `站点名称|站点方位|<two-digit stop number>`
///
/// Insert strips / 插片:
/// Side A: `路段名_站点名称_站点方位_尺寸_A|B|C` (5 components)
/// Side B: `路段名_站点名称_站点方位_一|二|三_序号_尺寸` (6 components)
/// A/B/C correspond to 一/二/三; the letter form always denotes 序号 001.
/// Key: `站点名称|站点方位|插片<index>|<three-digit 序号>`
///
/// All keys intentionally omit 路段名 and 尺寸 per product requirements.
pub fn canonical_pairing_key(file_name: &str) -> Option<String> {
    let stem = strip_png_extension(file_name)?;
    let parts: Vec<&str> = stem.split('_').collect();
    match parts.len() {
        5 => insert_strip_letter_key(&parts),
        6 => bus_stop_key(&parts).or_else(|| insert_strip_numeral_key(&parts)),
        _ => None,
    }
}

fn bus_stop_key(parts: &[&str]) -> Option<String> {
    let station_name = parts[1].trim();
    let direction = parts[2].trim();
    let number_field = parts[3].trim();
    let face_field = parts[5].trim();

    if station_name.is_empty() || direction.is_empty() || number_field.is_empty() {
        return None;
    }

    let raw_number: u32 = number_field.parse().ok()?;

    let stop_number = match face_field {
        "正" | "反" => raw_number,
        "A" | "a" => raw_number.checked_sub(1)?.checked_mul(2)?.checked_add(1)?,
        "B" | "b" => raw_number.checked_sub(1)?.checked_mul(2)?.checked_add(2)?,
        _ => return None,
    };

    if stop_number == 0 {
        return None;
    }

    Some(format!("{station_name}|{direction}|{stop_number:02}"))
}

/// `路段名_站点名称_站点方位_尺寸_A|B|C` — the letter picks the strip index
/// and implicitly denotes 序号 001.
fn insert_strip_letter_key(parts: &[&str]) -> Option<String> {
    let station_name = parts[1].trim();
    let direction = parts[2].trim();
    let size = parts[3].trim();
    let index = insert_strip_index_from_letter(parts[4].trim())?;

    if station_name.is_empty() || direction.is_empty() || !is_size_token(size) {
        return None;
    }

    Some(insert_strip_key(station_name, direction, index, 1))
}

/// `路段名_站点名称_站点方位_一|二|三_序号_尺寸`.
fn insert_strip_numeral_key(parts: &[&str]) -> Option<String> {
    let station_name = parts[1].trim();
    let direction = parts[2].trim();
    let index = insert_strip_index_from_numeral(parts[3].trim())?;
    let sequence: u32 = parts[4].trim().parse().ok()?;
    let size = parts[5].trim();

    if station_name.is_empty() || direction.is_empty() || !is_size_token(size) {
        return None;
    }

    Some(insert_strip_key(station_name, direction, index, sequence))
}

fn insert_strip_key(station_name: &str, direction: &str, index: u32, sequence: u32) -> String {
    format!("{station_name}|{direction}|插片{index}|{sequence:03}")
}

fn insert_strip_index_from_letter(letter: &str) -> Option<u32> {
    match letter {
        "A" | "a" => Some(1),
        "B" | "b" => Some(2),
        "C" | "c" => Some(3),
        _ => None,
    }
}

fn insert_strip_index_from_numeral(numeral: &str) -> Option<u32> {
    match numeral {
        "一" => Some(1),
        "二" => Some(2),
        "三" => Some(3),
        _ => None,
    }
}

/// `尺寸` must look like `<digits>x<digits>` (e.g. `195x920`) — for the
/// 5-component letter form this is the main guard against misclassifying
/// unrelated underscore-separated names.
fn is_size_token(value: &str) -> bool {
    let mut halves = value.splitn(2, ['x', 'X']);
    match (halves.next(), halves.next()) {
        (Some(width), Some(height)) => {
            !width.is_empty()
                && !height.is_empty()
                && width.bytes().all(|byte| byte.is_ascii_digit())
                && height.bytes().all(|byte| byte.is_ascii_digit())
        }
        _ => false,
    }
}

fn strip_png_extension(file_name: &str) -> Option<&str> {
    let dot = file_name.rfind('.')?;
    let (stem, ext) = file_name.split_at(dot);
    if ext.eq_ignore_ascii_case(".png") && !stem.is_empty() {
        Some(stem)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::canonical_pairing_key;

    #[test]
    fn side_a_zheng_uses_stop_number_directly() {
        assert_eq!(
            canonical_pairing_key("前进一路_新安公园地铁站_西_1_1350x2060_正.png"),
            Some("新安公园地铁站|西|01".to_string())
        );
    }

    #[test]
    fn side_a_fan_uses_stop_number_directly() {
        assert_eq!(
            canonical_pairing_key("前进一路_新安公园地铁站_西_2_1350x2060_反.png"),
            Some("新安公园地铁站|西|02".to_string())
        );
    }

    #[test]
    fn side_b_rack_01_a_converts_to_stop_01() {
        assert_eq!(
            canonical_pairing_key("前进一路_新安公园地铁站_西_01_1350x2060_A.png"),
            Some("新安公园地铁站|西|01".to_string())
        );
    }

    #[test]
    fn side_b_rack_01_b_converts_to_stop_02() {
        assert_eq!(
            canonical_pairing_key("前进一路_新安公园地铁站_西_01_1350x2060_B.png"),
            Some("新安公园地铁站|西|02".to_string())
        );
    }

    #[test]
    fn side_b_rack_02_a_converts_to_stop_03() {
        assert_eq!(
            canonical_pairing_key("X_Y_北_02_W_A.png"),
            Some("Y|北|03".to_string())
        );
    }

    #[test]
    fn side_b_rack_02_b_converts_to_stop_04() {
        assert_eq!(
            canonical_pairing_key("X_Y_北_02_W_B.png"),
            Some("Y|北|04".to_string())
        );
    }

    #[test]
    fn cross_format_pairs_produce_identical_keys() {
        let side_a = canonical_pairing_key("前进一路_新安公园地铁站_西_1_1350x2060_正.png");
        let side_b = canonical_pairing_key("前进一路_新安公园地铁站_西_01_1350x2060_A.png");
        assert!(side_a.is_some());
        assert_eq!(side_a, side_b);
    }

    #[test]
    fn different_routes_or_sizes_do_not_affect_key() {
        let left = canonical_pairing_key("路A_站_南_03_100x200_正.png");
        let right = canonical_pairing_key("路B_站_南_03_999x999_正.png");
        assert_eq!(left, right);
    }

    #[test]
    fn lowercase_face_letters_are_accepted() {
        assert_eq!(
            canonical_pairing_key("R_S_E_01_W_a.png"),
            Some("S|E|01".to_string())
        );
        assert_eq!(
            canonical_pairing_key("R_S_E_01_W_b.png"),
            Some("S|E|02".to_string())
        );
    }

    #[test]
    fn case_insensitive_png_extension() {
        assert_eq!(
            canonical_pairing_key("R_S_E_1_W_正.PNG"),
            Some("S|E|01".to_string())
        );
    }

    #[test]
    fn returns_none_for_too_few_components() {
        assert_eq!(canonical_pairing_key("S_E_1_W_正.png"), None);
    }

    #[test]
    fn returns_none_for_too_many_components() {
        assert_eq!(canonical_pairing_key("R_S_E_1_W_正_extra.png"), None);
    }

    #[test]
    fn returns_none_for_copy_suffix_in_parens() {
        assert_eq!(
            canonical_pairing_key("前进一路_新安公园地铁站_西_1_1350x2060_正 (1).png"),
            None
        );
    }

    #[test]
    fn returns_none_for_non_numeric_number_field() {
        assert_eq!(canonical_pairing_key("R_S_E_ab_W_A.png"), None);
    }

    #[test]
    fn returns_none_for_unknown_face_token() {
        assert_eq!(canonical_pairing_key("R_S_E_1_W_C.png"), None);
    }

    #[test]
    fn returns_none_for_non_png_extension() {
        assert_eq!(canonical_pairing_key("R_S_E_1_W_A.jpg"), None);
        assert_eq!(canonical_pairing_key("R_S_E_1_W_A"), None);
    }

    #[test]
    fn returns_none_for_rack_zero() {
        // (0-1) underflow guarded by checked_sub
        assert_eq!(canonical_pairing_key("R_S_E_0_W_A.png"), None);
    }

    #[test]
    fn returns_none_for_empty_required_components() {
        assert_eq!(canonical_pairing_key("R__E_1_W_A.png"), None);
        assert_eq!(canonical_pairing_key("R_S__1_W_A.png"), None);
        assert_eq!(canonical_pairing_key("R_S_E__W_A.png"), None);
    }

    #[test]
    fn pad_width_handles_three_digit_stop_numbers() {
        assert_eq!(
            canonical_pairing_key("R_S_E_100_W_正.png"),
            Some("S|E|100".to_string())
        );
    }

    #[test]
    fn insert_strip_letter_c_matches_numeral_three_001() {
        let letter = canonical_pairing_key("平冠道_平冠道_东_195x920_C.png");
        let numeral = canonical_pairing_key("平冠道_平冠道_东_三_001_195x920.png");
        assert_eq!(letter, Some("平冠道|东|插片3|001".to_string()));
        assert_eq!(letter, numeral);
    }

    #[test]
    fn insert_strip_letter_a_matches_numeral_one_001() {
        let letter = canonical_pairing_key("平冠道_平冠道_东_195x920_A.png");
        let numeral = canonical_pairing_key("平冠道_平冠道_东_一_001_195x920.png");
        assert_eq!(letter, Some("平冠道|东|插片1|001".to_string()));
        assert_eq!(letter, numeral);
    }

    #[test]
    fn insert_strip_letter_b_matches_numeral_two_001() {
        let letter = canonical_pairing_key("平冠道_平冠道_东_195x920_B.png");
        let numeral = canonical_pairing_key("平冠道_平冠道_东_二_001_195x920.png");
        assert_eq!(letter, Some("平冠道|东|插片2|001".to_string()));
        assert_eq!(letter, numeral);
    }

    #[test]
    fn insert_strip_lowercase_letters_are_accepted() {
        assert_eq!(
            canonical_pairing_key("平冠道_平冠道_东_195x920_c.png"),
            Some("平冠道|东|插片3|001".to_string())
        );
    }

    #[test]
    fn insert_strip_sequence_is_zero_padded_in_key() {
        assert_eq!(
            canonical_pairing_key("平冠道_平冠道_东_三_1_195x920.png"),
            canonical_pairing_key("平冠道_平冠道_东_195x920_C.png")
        );
    }

    #[test]
    fn insert_strip_sequence_002_does_not_match_letter_form() {
        let numeral = canonical_pairing_key("平冠道_平冠道_东_三_002_195x920.png");
        let letter = canonical_pairing_key("平冠道_平冠道_东_195x920_C.png");
        assert_eq!(numeral, Some("平冠道|东|插片3|002".to_string()));
        assert_ne!(numeral, letter);
    }

    #[test]
    fn insert_strip_ignores_route_and_size_differences() {
        assert_eq!(
            canonical_pairing_key("路A_站_南_100x200_C.png"),
            canonical_pairing_key("路B_站_南_三_001_999x999.png")
        );
    }

    #[test]
    fn insert_strip_unknown_letter_returns_none() {
        assert_eq!(canonical_pairing_key("平冠道_平冠道_东_195x920_D.png"), None);
    }

    #[test]
    fn insert_strip_unknown_numeral_returns_none() {
        assert_eq!(
            canonical_pairing_key("平冠道_平冠道_东_四_001_195x920.png"),
            None
        );
    }

    #[test]
    fn insert_strip_letter_form_requires_size_token() {
        assert_eq!(canonical_pairing_key("R_S_E_W_A.png"), None);
        assert_eq!(canonical_pairing_key("R_S_E_12x_A.png"), None);
    }

    #[test]
    fn insert_strip_numeral_form_requires_numeric_sequence() {
        assert_eq!(canonical_pairing_key("R_S_E_一_abc_100x200.png"), None);
    }

    #[test]
    fn insert_strip_key_does_not_collide_with_stop_key() {
        let stop = canonical_pairing_key("R_S_E_3_100x200_正.png");
        let strip = canonical_pairing_key("R_S_E_100x200_C.png");
        assert!(stop.is_some());
        assert!(strip.is_some());
        assert_ne!(stop, strip);
    }

    #[test]
    fn insert_strip_returns_none_for_empty_required_components() {
        assert_eq!(canonical_pairing_key("R__E_100x200_C.png"), None);
        assert_eq!(canonical_pairing_key("R_S__100x200_C.png"), None);
        assert_eq!(canonical_pairing_key("R__E_三_001_100x200.png"), None);
        assert_eq!(canonical_pairing_key("R_S__三_001_100x200.png"), None);
    }
}
