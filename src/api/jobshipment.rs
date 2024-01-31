
//! Job and shipment

use std::fmt::{self, Display, Formatter};
use std::str::FromStr;
use std::sync::LazyLock;

static JOB_SHIPMENT_PATTERN: LazyLock<regex::Regex> = LazyLock::new(|| regex::Regex::new(r"^(\d{7}[[:alpha:]])-(\d+)$").expect("failed to build JOB_SHIPMENT_PATTERN regex"));

/// Job number (with structure letter) and shipment
#[derive(Debug, Clone)]
pub struct JobShipment {
    /// Job Number
    job: String,
    /// Shipment
    shipment: u32
}

#[derive(Debug)]
pub enum JobShipmentParseError {
    // InvalidJob,
    // MissingStructureLetter,
    InvalidShipment,
    ExpectedPatternMismatch,
}

impl Display for JobShipmentParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidShipment => write!(f, "shipment is expected to be a number"),
            Self::ExpectedPatternMismatch => write!(f, "job-shipment does not match expected pattern"),
        }
    }
}

impl FromStr for JobShipment {
    type Err = JobShipmentParseError;
    
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        log::trace!("Parsing JobShipment <FromStr> {}", value);

        match JOB_SHIPMENT_PATTERN.captures(value) {
            Some(cap) => {
                let caps: [&str; 2] = cap.extract().1;
                
                // is this needed? regex might handle parsing errors
                match caps[1].parse() {
                    Ok(shipment) => Ok( Self { job: caps[0].to_uppercase().into(), shipment } ),
                    Err(_) => Err(JobShipmentParseError::InvalidShipment)
                }
            },
            None => Err(JobShipmentParseError::ExpectedPatternMismatch)
        }
    }
}

impl From<String> for JobShipment {
    fn from(value: String) -> Self {
        log::trace!("Parsing JobShipment <From> {}", value);
        
        Self::from_str(&value).unwrap()
    }
}

impl Display for JobShipment {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}", self.job, self.shipment)
    }
}
