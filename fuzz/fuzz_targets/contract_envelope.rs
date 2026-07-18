#![no_main]

use libfuzzer_sys::fuzz_target;
use serde_json::Value;
use ste_contracts::{
    CaptureHealthV1, ContractEnvelopeV1, ObservationWindowClosedV1, ValidatedCsiFrameV1,
};

fuzz_target!(|data: &[u8]| {
    // These are all future external parser boundaries. Rejection is expected;
    // panics, hangs, and unbounded allocation are not.
    let _ = serde_json::from_slice::<ContractEnvelopeV1<Value>>(data);
    let _ = serde_json::from_slice::<ValidatedCsiFrameV1>(data);
    let _ = serde_json::from_slice::<CaptureHealthV1>(data);
    let _ = serde_json::from_slice::<ObservationWindowClosedV1>(data);
});
