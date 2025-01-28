pub fn get_bytes(bytes: u64) -> String {
    if bytes < 1024u64.pow(1) {
        format!("{} B", bytes)
    } else if bytes < 1024u64.pow(2) {
        format!("{:.2} KiB", bytes as f64 / 1024f64.powf(1.))
    } else if bytes < 1024u64.pow(3) {
        format!("{:.2} MiB", bytes as f64 / 1024f64.powf(2.))
    } else {
        format!("{:.2} GiB", bytes as f64 / 1024f64.powf(3.))
    }
}
