//! Gyrognome-only metadata appended to exported desktop saves.
//!
//! pq.exe 6.4.4 reads exactly its fixed component list and ignores the rest
//! of the inflated stream, so the block survives only Gyrognome round trips.

use ring::digest::{SHA256, digest};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::compatibility::{DesktopImportMetadata, DesktopRandomState};

pub const MAGIC: &[u8] = b"GYROGNOME-META";
pub const MAX_BLOCK_BYTES: usize = 64 * 1024;
const VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopRestoredMetadata {
    pub import_metadata: DesktopImportMetadata,
    pub random: DesktopRandomState,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DesktopMetadataError {
    #[error("Gyrognome metadata block is truncated or has trailing bytes")]
    Framing,
    #[error("Gyrognome metadata block exceeds {MAX_BLOCK_BYTES} bytes")]
    TooLarge,
    #[error("Gyrognome metadata block is invalid or has an unsupported version")]
    Invalid,
    #[error("Gyrognome metadata block does not match the save's desktop state")]
    DigestMismatch,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Block {
    version: u32,
    components_sha256: String,
    import_metadata: DesktopImportMetadata,
    random: DesktopRandomState,
}

fn components_digest(components: &[u8]) -> String {
    digest(&SHA256, components)
        .as_ref()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// Encodes the block for the given inflated component bytes.
pub fn encode(
    components: &[u8],
    metadata: &DesktopRestoredMetadata,
) -> Result<Vec<u8>, DesktopMetadataError> {
    let json = serde_json::to_vec(&Block {
        version: VERSION,
        components_sha256: components_digest(components),
        import_metadata: metadata.import_metadata.clone(),
        random: metadata.random,
    })
    .map_err(|_| DesktopMetadataError::Invalid)?;
    if json.len() > MAX_BLOCK_BYTES {
        return Err(DesktopMetadataError::TooLarge);
    }
    let mut out = MAGIC.to_vec();
    out.extend_from_slice(&(json.len() as u32).to_le_bytes());
    out.extend(json);
    Ok(out)
}

/// Decodes a block that starts with [`MAGIC`] and must end exactly at the end
/// of `block`, verifying it against the preceding component bytes.
pub fn decode(
    components: &[u8],
    block: &[u8],
) -> Result<DesktopRestoredMetadata, DesktopMetadataError> {
    let rest = block
        .strip_prefix(MAGIC)
        .ok_or(DesktopMetadataError::Framing)?;
    let (length, json) = rest
        .split_first_chunk::<4>()
        .ok_or(DesktopMetadataError::Framing)?;
    let length = u32::from_le_bytes(*length) as usize;
    if length > MAX_BLOCK_BYTES {
        return Err(DesktopMetadataError::TooLarge);
    }
    if json.len() != length {
        return Err(DesktopMetadataError::Framing);
    }
    let block: Block = serde_json::from_slice(json).map_err(|_| DesktopMetadataError::Invalid)?;
    if block.version != VERSION {
        return Err(DesktopMetadataError::Invalid);
    }
    if block.components_sha256 != components_digest(components) {
        return Err(DesktopMetadataError::DigestMismatch);
    }
    Ok(DesktopRestoredMetadata {
        import_metadata: block.import_metadata,
        random: block.random,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{compatibility::DesktopAdvancementProvenance, desktop_save::DesktopAdaptations};

    fn metadata() -> DesktopRestoredMetadata {
        let mut import_metadata = DesktopImportMetadata::from_validated(&DesktopAdaptations {
            legacy_prologue_62: true,
            legacy_quest_placeholder: false,
            spelling_patch_applied: false,
        });
        import_metadata.measured_since_import.tasks_completed = 12;
        import_metadata.measured_since_import.elapsed_milliseconds = 34_000;
        import_metadata.advancement_provenance = DesktopAdvancementProvenance::LocalOnly;
        DesktopRestoredMetadata {
            import_metadata,
            random: DesktopRandomState(0xdead_beef),
        }
    }

    #[test]
    fn round_trips_against_matching_components() {
        let block = encode(b"TPF0components", &metadata()).unwrap();
        assert!(block.starts_with(MAGIC));
        assert_eq!(decode(b"TPF0components", &block).unwrap(), metadata());
    }

    #[test]
    fn rejects_mismatch_framing_size_and_version() {
        let block = encode(b"TPF0components", &metadata()).unwrap();
        assert_eq!(
            decode(b"TPF0changed", &block),
            Err(DesktopMetadataError::DigestMismatch)
        );
        let mut trailing = block.clone();
        trailing.push(0);
        assert_eq!(
            decode(b"TPF0components", &trailing),
            Err(DesktopMetadataError::Framing)
        );
        assert_eq!(
            decode(b"TPF0components", &block[..block.len() - 1]),
            Err(DesktopMetadataError::Framing)
        );
        let mut oversized = MAGIC.to_vec();
        oversized.extend_from_slice(&((MAX_BLOCK_BYTES + 1) as u32).to_le_bytes());
        assert_eq!(
            decode(b"TPF0components", &oversized),
            Err(DesktopMetadataError::TooLarge)
        );
        let json = String::from_utf8(block[MAGIC.len() + 4..].to_vec())
            .unwrap()
            .replace("\"version\":1", "\"version\":2");
        let mut future = MAGIC.to_vec();
        future.extend_from_slice(&(json.len() as u32).to_le_bytes());
        future.extend_from_slice(json.as_bytes());
        assert_eq!(
            decode(b"TPF0components", &future),
            Err(DesktopMetadataError::Invalid)
        );
    }
}
