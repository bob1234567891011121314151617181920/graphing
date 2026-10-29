const SHORT_NUMBER_SUFFIXES: [&str; 6] = ["K", "M", "B", "T", "Qd", "Qn"];

fn conver_to_short_number(num: f64) -> String {
    if num.abs() < 1000.0 {
        return format!("{num}");
    }

    let i = (num.abs().log10() / 3.0).floor() as usize;
    let short = num / 10f64.powi(i as i32 * 3);
    format!("{short:.2}{}", SHORT_NUMBER_SUFFIXES[i - 1])
}

pub fn convert_to_regular_number(text: &str) -> f64 {
    for (i, suffix) in SHORT_NUMBER_SUFFIXES.iter().enumerate() {
        if let Some(number) = text.strip_suffix(suffix) {
            let value: f64 = number
                .parse()
                .unwrap_or_else(|e| panic!("can't parse {text:?}: {e}"));
            return value * 10f64.powi((i as i32 + 1) * 3);
        }
    }

    text.parse()
        .unwrap_or_else(|e| panic!("can't parse {text:?}: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_short() {
        assert_eq!(conver_to_short_number(850.0), "850");
        assert_eq!(conver_to_short_number(1250.0), "1.25K");
        assert_eq!(conver_to_short_number(1_500_000.0), "1.50M");
        assert_eq!(conver_to_short_number(-2500.0), "-2.50K");
    }

    #[test]
    fn to_regular() {
        assert_eq!(convert_to_regular_number("850"), 850.0);
        assert_eq!(convert_to_regular_number("1.25K"), 1250.0);
        assert_eq!(convert_to_regular_number("1.5M"), 1_500_000.0);
        assert_eq!(convert_to_regular_number("-2.5K"), -2500.0);
    }

    #[test]
    fn round_trip() {
        let text = conver_to_short_number(1250.0);
        assert_eq!(convert_to_regular_number(&text), 1250.0);
    }
}
