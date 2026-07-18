use schemars::schema_for;
use serde_json::{json, to_writer_pretty};
use ste_contracts::{
    CaptureHealthV1, ContractEnvelopeV1, DisplayProjectionV1, ObservationWindowClosedV1,
    PhysiologyEvidenceUpdatedV1, ValidatedCsiFrameV1,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bundle = json!({
        "capture_health_event_v1": schema_for!(ContractEnvelopeV1<CaptureHealthV1>),
        "validated_csi_frame_v1": schema_for!(ValidatedCsiFrameV1),
        "observation_window_closed_v1": schema_for!(ObservationWindowClosedV1),
        "physiology_evidence_updated_v1": schema_for!(PhysiologyEvidenceUpdatedV1),
        "display_projection_v1": schema_for!(DisplayProjectionV1),
    });
    to_writer_pretty(std::io::stdout(), &bundle)?;
    println!();
    Ok(())
}
