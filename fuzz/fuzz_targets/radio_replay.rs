#![no_main]

use libfuzzer_sys::fuzz_target;
use ste_radio_acquisition::replay::{ReplayLimits, parse_pcap, parse_rvcsi};

fuzz_target!(|data: &[u8]| {
    // Tight fuzz budgets exercise rejection logic without allowing the input to
    // drive unbounded frame/sample allocation.
    let limits = ReplayLimits {
        max_input_bytes: 1 << 20,
        max_frames: 4_096,
        max_subcarriers: 1_024,
        max_record_bytes: 64 * 1024,
    };
    let _ = parse_rvcsi(data, limits);
    let _ = parse_pcap(data, limits);
});
