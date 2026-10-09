use std::{
    fs,
    io::{self, Read},
    path::Path,
    str,
};

use base64::{Engine, engine::general_purpose::STANDARD};
use flate2::read::ZlibDecoder;
use serde::Serialize;
use serde_json::Value;
use thiserror::Error;

use crate::{
    compatibility::{CompatibilityProfile, DesktopCanonicalState, DesktopImportMetadata},
    desktop_eligibility::{
        DesktopEligibilityInput, DesktopOnlineOperation, DesktopOperationEligibility,
        evaluate_production_desktop_eligibility,
    },
    desktop_save::{
        DesktopDocument, DesktopMappingError, DesktopParseError, DesktopValidatedSave,
        DesktopValidationError, map_desktop_document, parse_components, validate_desktop_save,
    },
    state::Character,
};

pub const MAX_DESKTOP_COMPRESSED_BYTES: usize = 8 * 1024 * 1024;
pub const MAX_DESKTOP_INFLATED_BYTES: usize = 64 * 1024 * 1024;

#[derive(Debug, Error)]
pub enum SaveError {
    #[error("could not read save: {0}")]
    Read(#[from] std::io::Error),
    #[error("save is neither a supported desktop container nor UTF-8 browser text")]
    Text(#[from] str::Utf8Error),
    #[error("save is not valid Base64")]
    Base64(#[from] base64::DecodeError),
    #[error("save JSON is invalid: {0}")]
    Json(#[from] serde_json::Error),
    #[error("desktop save exceeds the {MAX_DESKTOP_COMPRESSED_BYTES}-byte compressed input limit")]
    DesktopCompressedTooLarge,
    #[error("desktop save could not be inflated as zlib: {0}")]
    DesktopZlib(#[source] io::Error),
    #[error("desktop save exceeds the {MAX_DESKTOP_INFLATED_BYTES}-byte inflated output limit")]
    DesktopInflatedTooLarge,
    #[error("inflated desktop save does not begin with a Delphi TPF0 component")]
    DesktopMissingComponentHeader,
    #[error(transparent)]
    DesktopStructure(#[from] DesktopParseError),
    #[error(transparent)]
    DesktopMapping(#[from] DesktopMappingError),
    #[error(transparent)]
    DesktopValidation(#[from] DesktopValidationError),
    #[error("desktop save requires the profile-aware import API")]
    DesktopRequiresProfileAwareImport,
    #[error("save is missing or has an invalid required field: {0}")]
    RequiredField(String),
}

impl SaveError {
    pub(crate) fn invalid(field: impl Into<String>) -> Self {
        Self::RequiredField(field.into())
    }
}

pub fn import_file(path: &Path) -> Result<Character, SaveError> {
    import_bytes(&fs::read(path)?)
}

pub fn import_bytes(bytes: &[u8]) -> Result<Character, SaveError> {
    match import_supported_bytes(bytes)? {
        ImportedSave::Browser(character) => Ok(character),
        ImportedSave::Desktop(_) => Err(SaveError::DesktopRequiresProfileAwareImport),
    }
}

#[derive(Debug)]
pub enum ImportedSave {
    Browser(Character),
    Desktop(DesktopValidatedSave),
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopSaveInspection {
    pub target_profile: CompatibilityProfile,
    pub state: DesktopCanonicalState,
    pub import_metadata: DesktopImportMetadata,
    pub online_eligibility: Vec<DesktopOperationEligibility>,
}

pub fn import_supported_file(path: &Path) -> Result<ImportedSave, SaveError> {
    import_supported_bytes(&fs::read(path)?)
}

pub fn import_supported_bytes(bytes: &[u8]) -> Result<ImportedSave, SaveError> {
    match decode_bytes(bytes)? {
        DecodedSave::Browser(document) => {
            Character::from_document(document).map(ImportedSave::Browser)
        }
        DecodedSave::Desktop(document) => {
            let mapped = map_desktop_document(&document)?;
            Ok(ImportedSave::Desktop(validate_desktop_save(&mapped)?))
        }
    }
}

pub fn inspect_desktop(save: &DesktopValidatedSave) -> DesktopSaveInspection {
    let state = DesktopCanonicalState::from(save);
    let import_metadata = DesktopImportMetadata::from_validated(&save.adaptations);
    let operations = [
        DesktopOnlineOperation::AutomaticLevel,
        DesktopOnlineOperation::AutomaticAct,
        DesktopOnlineOperation::ManualBrag,
        DesktopOnlineOperation::Motto,
        DesktopOnlineOperation::Guild,
    ];
    let online_eligibility = operations
        .into_iter()
        .map(|operation| {
            let input = DesktopEligibilityInput {
                profile: CompatibilityProfile::Desktop644,
                source_format: import_metadata.provenance.source_format,
                layout: import_metadata.provenance.recognized_layout,
                adaptations: &import_metadata.provenance.adaptations,
                advancement: import_metadata.advancement_provenance,
                passkey: save.private.passkey,
                realm: &save.private.realm,
                endpoint: &save.private.endpoint,
                account: &save.private.account,
                password: &save.private.password,
                encoding_supported: crate::desktop_eligibility::desktop_request_text_is_ascii(
                    &state,
                    &[
                        &save.private.realm,
                        &save.private.endpoint,
                        &save.private.account,
                        &save.private.password,
                    ],
                ),
                operation,
            };
            DesktopOperationEligibility {
                operation,
                decision: evaluate_production_desktop_eligibility(&input),
            }
        })
        .collect();
    DesktopSaveInspection {
        target_profile: CompatibilityProfile::Desktop644,
        state,
        import_metadata,
        online_eligibility,
    }
}

pub fn import_text(text: &str) -> Result<Character, SaveError> {
    Character::from_document(decode_browser(text)?)
}

pub fn export(character: &Character) -> Result<String, SaveError> {
    Ok(STANDARD.encode(serde_json::to_vec(&character.document)?))
}

pub(crate) fn encode_browser_document(document: &Value) -> Result<String, serde_json::Error> {
    Ok(STANDARD.encode(serde_json::to_vec(document)?))
}

#[derive(Debug, PartialEq)]
pub(crate) enum DecodedSave {
    Browser(Value),
    Desktop(DesktopDocument),
}

pub(crate) fn decode_bytes(bytes: &[u8]) -> Result<DecodedSave, SaveError> {
    if is_zlib(bytes) {
        decode_desktop(bytes).map(DecodedSave::Desktop)
    } else {
        decode_browser(str::from_utf8(bytes)?).map(DecodedSave::Browser)
    }
}

fn decode_browser(text: &str) -> Result<Value, SaveError> {
    let decoded = STANDARD.decode(text.trim())?;
    Ok(serde_json::from_slice(&decoded)?)
}

fn is_zlib(bytes: &[u8]) -> bool {
    let [compression, flags, ..] = bytes else {
        return false;
    };
    compression & 0x0f == 8
        && compression >> 4 <= 7
        && (u16::from(*compression) << 8 | u16::from(*flags)) % 31 == 0
}

fn decode_desktop(bytes: &[u8]) -> Result<DesktopDocument, SaveError> {
    if bytes.len() > MAX_DESKTOP_COMPRESSED_BYTES {
        return Err(SaveError::DesktopCompressedTooLarge);
    }
    let decoder = ZlibDecoder::new(bytes);
    let mut limited = decoder.take((MAX_DESKTOP_INFLATED_BYTES + 1) as u64);
    let mut inflated = Vec::new();
    limited
        .read_to_end(&mut inflated)
        .map_err(SaveError::DesktopZlib)?;
    if inflated.len() > MAX_DESKTOP_INFLATED_BYTES {
        return Err(SaveError::DesktopInflatedTooLarge);
    }
    if !inflated.starts_with(b"TPF0") {
        return Err(SaveError::DesktopMissingComponentHeader);
    }
    Ok(parse_components(&inflated)?)
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        io::Write,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use flate2::{Compression, write::ZlibEncoder};
    use serde_json::json;

    use super::*;

    fn desktop_bytes(payload: &[u8]) -> Vec<u8> {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(payload).unwrap();
        encoder.finish().unwrap()
    }

    fn component_bytes(name: &str) -> Vec<u8> {
        let mut payload = b"TPF0".to_vec();
        payload.push(6);
        payload.extend_from_slice(b"TLabel");
        payload.push(name.len() as u8);
        payload.extend_from_slice(name.as_bytes());
        payload.extend([0, 0]);
        payload
    }

    fn temporary_path(extension: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "gyrognome-save-dispatch-{}-{nonce}.{extension}",
            std::process::id()
        ))
    }

    #[test]
    fn browser_bytes_preserve_existing_decode_and_export() {
        let document = json!({"hello": "browser"});
        let encoded = STANDARD.encode(serde_json::to_vec(&document).unwrap());
        assert_eq!(
            decode_bytes(encoded.as_bytes()).unwrap(),
            DecodedSave::Browser(document)
        );
    }

    #[test]
    fn desktop_dispatch_is_content_based_for_pq_and_bak() {
        let payload = component_bytes("Synthetic");
        let compressed = desktop_bytes(&payload);
        for extension in ["pq", "bak"] {
            let path = temporary_path(extension);
            fs::write(&path, &compressed).unwrap();
            let decoded = decode_bytes(&fs::read(&path).unwrap()).unwrap();
            fs::remove_file(path).unwrap();
            let DecodedSave::Desktop(document) = decoded else {
                panic!("expected desktop content");
            };
            assert_eq!(document.components[0].name, "Synthetic");
        }
    }

    #[test]
    fn extensions_do_not_override_content_detection() {
        let document = json!({"hello": "browser"});
        let encoded = STANDARD.encode(serde_json::to_vec(&document).unwrap());
        for extension in ["pq", "bak"] {
            let path = temporary_path(extension);
            fs::write(&path, &encoded).unwrap();
            let decoded = decode_bytes(&fs::read(&path).unwrap()).unwrap();
            fs::remove_file(path).unwrap();
            assert_eq!(decoded, DecodedSave::Browser(document.clone()));
        }
    }

    #[test]
    fn malformed_recognized_zlib_does_not_fall_back_to_browser_decode() {
        let error = decode_bytes(&[0x78, 0x9c, 0x00]).unwrap_err();
        assert!(matches!(error, SaveError::DesktopZlib(_)));
    }

    #[test]
    fn desktop_input_and_output_limits_are_enforced() {
        let oversized_input = vec![0; MAX_DESKTOP_COMPRESSED_BYTES + 1];
        let mut recognized = oversized_input;
        recognized[0] = 0x78;
        recognized[1] = 0x9c;
        assert!(matches!(
            decode_bytes(&recognized),
            Err(SaveError::DesktopCompressedTooLarge)
        ));

        let compressed = desktop_bytes(&vec![b'A'; MAX_DESKTOP_INFLATED_BYTES + 1]);
        assert!(matches!(
            decode_bytes(&compressed),
            Err(SaveError::DesktopInflatedTooLarge)
        ));
    }

    #[test]
    fn valid_zlib_without_tpf0_is_a_desktop_format_error() {
        let compressed = desktop_bytes(b"not a Delphi component");
        assert!(matches!(
            decode_bytes(&compressed),
            Err(SaveError::DesktopMissingComponentHeader)
        ));
    }
}
