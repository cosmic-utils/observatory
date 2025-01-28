pub fn format_number(num: f64) -> String {
    if num.fract() == 0.0 {
        // If it's a whole number, format with no decimals
        format!("{:.0}", num)
    } else {
        // If it has decimals, format with up to 2 decimal places
        format!("{:.2}", num)
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    }
}

pub fn get_bytes(bytes: u64) -> String {
    if bytes < 1024u64.pow(1) {
        format!("{} B", bytes)
    } else if bytes < 1024u64.pow(2) {
        format!("{} KiB", format_number(bytes as f64 / 1024f64.powf(1.)))
    } else if bytes < 1024u64.pow(3) {
        format!("{} MiB", format_number(bytes as f64 / 1024f64.powf(2.)))
    } else {
        format!("{} GiB", format_number(bytes as f64 / 1024f64.powf(3.)))
    }
}
