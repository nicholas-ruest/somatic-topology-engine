//! Preview-bound, checksummed local support bundle manifests.
use crate::{Record, RedactionSchema};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fmt, fmt::Write as _};
/// Manifest entry.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct BundleEntry {
    /// Logical name.
    pub name: String,
    /// SHA-256.
    pub checksum: String,
    /// Bytes.
    pub size: usize,
}
/// Support bundle manifest.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct BundleManifest {
    /// Schema version.
    pub version: u16,
    /// Redacted entries.
    pub entries: Vec<BundleEntry>,
}
/// Exact preview authorization token.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BundlePreview {
    digest: String,
    /// Manifest shown to user.
    pub manifest: BundleManifest,
}
/// Bundle failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BundleError {
    /// Preview does not match current content.
    PreviewMismatch,
    /// Serialization failure.
    Encoding,
}
impl fmt::Display for BundleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
/// Local support bundle builder.
pub struct SupportBundleBuilder<'a> {
    schema: &'a RedactionSchema,
    records: Vec<Record>,
}
impl<'a> SupportBundleBuilder<'a> {
    /// Creates builder with mandatory schema.
    #[must_use]
    pub fn new(schema: &'a RedactionSchema) -> Self {
        Self {
            schema,
            records: Vec::new(),
        }
    }
    /// Adds structured record for redaction.
    pub fn add(&mut self, record: Record) {
        self.records.push(record);
    }
    fn materialize(&self) -> Result<(BundleManifest, BTreeMap<String, Vec<u8>>), BundleError> {
        let mut files = BTreeMap::new();
        for (i, r) in self.records.iter().enumerate() {
            let bytes =
                serde_json::to_vec(&self.schema.redact(r)).map_err(|_| BundleError::Encoding)?;
            files.insert(format!("record-{i}.json"), bytes);
        }
        let entries = files
            .iter()
            .map(|(n, b)| BundleEntry {
                name: n.clone(),
                checksum: hex(&Sha256::digest(b)),
                size: b.len(),
            })
            .collect();
        Ok((
            BundleManifest {
                version: 1,
                entries,
            },
            files,
        ))
    }
    /// Produces exact manifest preview.
    pub fn preview(&self) -> Result<BundlePreview, BundleError> {
        let (manifest, _) = self.materialize()?;
        let bytes = serde_json::to_vec(&manifest).map_err(|_| BundleError::Encoding)?;
        Ok(BundlePreview {
            digest: hex(&Sha256::digest(bytes)),
            manifest,
        })
    }
    /// Exports only after exact preview confirmation.
    pub fn export(
        &self,
        preview: &BundlePreview,
    ) -> Result<BTreeMap<String, Vec<u8>>, BundleError> {
        let (current, files) = self.materialize()?;
        let bytes = serde_json::to_vec(&current).map_err(|_| BundleError::Encoding)?;
        if hex(&Sha256::digest(bytes)) != preview.digest {
            return Err(BundleError::PreviewMismatch);
        }
        Ok(files)
    }
}
fn hex(b: &[u8]) -> String {
    let mut output = String::with_capacity(b.len() * 2);
    for byte in b {
        write!(output, "{byte:02x}").expect("writing to String cannot fail");
    }
    output
}
