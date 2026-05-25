use crate::ork::{OpenRocketFile, OrkError};
use crate::rkt::{RockSimFile, RktError};
use federated_rocket_core::component_tree::ComponentTree;
use std::path::Path;

// ============================================================================
// Format Detection
// ============================================================================

/// Supported rocket file formats.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RocketFileFormat {
    /// OpenRocket format (.ork) - ZIP archive containing XML
    OpenRocket,
    /// RockSim format (.rkt) - structured text format
    RockSim,
    /// RASAero format (.rse) - not yet implemented
    RASAero,
    /// RockSim XML variant (.rkt, RockSim v10+)
    RockSimXML,
}

/// Detect file format from the file extension.
///
/// Returns `None` if the format is not recognized.
pub fn detect_format(path: &Path) -> Option<RocketFileFormat> {
    match path.extension()?.to_str()?.to_lowercase().as_str() {
        "ork" => Some(RocketFileFormat::OpenRocket),
        "rkt" => Some(RocketFileFormat::RockSim),
        "rse" => Some(RocketFileFormat::RASAero),
        _ => None,
    }
}

// ============================================================================
// Unified Loading
// ============================================================================

/// Error type for format detection and loading.
#[derive(Debug, thiserror::Error)]
pub enum FileIoError {
    #[error("OpenRocket error: {0}")]
    Ork(#[from] OrkError),
    #[error("RockSim error: {0}")]
    Rkt(#[from] RktError),
    #[error("Unsupported file format: {0}")]
    Unsupported(String),
}

impl From<Box<dyn std::error::Error>> for FileIoError {
    fn from(e: Box<dyn std::error::Error>) -> Self {
        FileIoError::Unsupported(e.to_string())
    }
}

/// Generic rocket file loader that auto-detects the format.
///
/// Uses [`detect_format`] to determine the file type and dispatches
/// to the appropriate parser.
///
/// # Errors
///
/// Returns `FileIoError::Unsupported` if the file format is not recognized
/// or not yet implemented.
pub fn load_rocket_file(path: &Path) -> Result<ComponentTree, FileIoError> {
    match detect_format(path) {
        Some(RocketFileFormat::OpenRocket) => {
            Ok(OpenRocketFile::load(path)?)
        }
        Some(RocketFileFormat::RockSim) => {
            Ok(RockSimFile::load(path)?)
        }
        Some(RocketFileFormat::RASAero) => {
            Err(FileIoError::Unsupported(
                "RASAero (.rse) format is not yet implemented".to_string(),
            ))
        }
        Some(RocketFileFormat::RockSimXML) => {
            // RockSim XML format would use a different parser
            // For now, delegate to the basic RockSim parser
            Ok(RockSimFile::load(path)?)
        }
        None => {
            Err(FileIoError::Unsupported(format!(
                "Unsupported file format: {}",
                path.display()
            )))
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_ork() {
        assert_eq!(
            detect_format(Path::new("rocket.ork")),
            Some(RocketFileFormat::OpenRocket)
        );
    }

    #[test]
    fn test_detect_rkt() {
        assert_eq!(
            detect_format(Path::new("rocket.rkt")),
            Some(RocketFileFormat::RockSim)
        );
    }

    #[test]
    fn test_detect_rse() {
        assert_eq!(
            detect_format(Path::new("rocket.rse")),
            Some(RocketFileFormat::RASAero)
        );
    }

    #[test]
    fn test_detect_case_insensitive() {
        assert_eq!(
            detect_format(Path::new("ROCKET.ORK")),
            Some(RocketFileFormat::OpenRocket)
        );
        assert_eq!(
            detect_format(Path::new("Rocket.Ork")),
            Some(RocketFileFormat::OpenRocket)
        );
    }

    #[test]
    fn test_detect_unknown() {
        assert_eq!(detect_format(Path::new("rocket.txt")), None);
        assert_eq!(detect_format(Path::new("rocket")), None);
        assert_eq!(detect_format(Path::new(".config")), None);
    }

    #[test]
    fn test_detect_no_extension() {
        assert_eq!(detect_format(Path::new("README")), None);
    }

    #[test]
    fn test_load_unsupported() {
        let result = load_rocket_file(Path::new("rocket.txt"));
        assert!(result.is_err());
        match result {
            Err(FileIoError::Unsupported(_)) => {} // expected
            _ => panic!("Expected Unsupported error"),
        }
    }

    #[test]
    fn test_load_rasaero() {
        let result = load_rocket_file(Path::new("rocket.rse"));
        assert!(result.is_err());
        match result {
            Err(FileIoError::Unsupported(msg)) => {
                assert!(msg.contains("not yet implemented"));
            }
            _ => panic!("Expected Unsupported error"),
        }
    }
}