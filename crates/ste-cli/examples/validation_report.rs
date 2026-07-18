//! Deterministic synthetic negative-control validation report.

use ste_experiment_validation::application::{EvidenceExportReader, ValidationStudyRepository};
use ste_experiment_validation::domain::{
    ArtifactDigest, Cohort, PromotionDecision, Protocol, StudyResult, ValidationStudy,
};
use ste_experiment_validation::{AtomicValidationRepository, ReproducibleValidationReport};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let digest = |byte| ArtifactDigest::new([byte; 32]);
    let protocol = Protocol::new("synthetic-negative-control", digest(1))?;
    let frozen =
        ValidationStudy::draft("phase-09-synthetic", protocol, Cohort::Synthetic)?.freeze(None)?;
    let run = frozen
        .start_run(
            "run-negative-control",
            digest(2),
            digest(3),
            digest(4),
            digest(5),
        )?
        .complete(StudyResult::rejected(
            "mandatory false-positive gate failed",
            digest(6),
        )?)?;
    let rejection = PromotionDecision::rejected(
        "presence-v1",
        frozen.id(),
        "mandatory false-positive gate failed",
        1,
    )?;
    let mut repository = AtomicValidationRepository::default();
    repository.save_frozen(&frozen)?;
    repository.append_run(&run)?;
    repository.append_promotion(&rejection)?;
    let export = repository.deidentified_export(frozen.id())?;
    let report = ReproducibleValidationReport::generate(&export)?;
    println!("{}  phase-09-validation.json", report.digest());
    println!("{}", String::from_utf8(report.bytes().to_vec())?);
    Ok(())
}
