
//! WBS element

use regex::Regex;
use std::sync::LazyLock;

// HD wbs element
static HD_WBS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"D-(\d{7})-(\d{5})").unwrap());
// old, non-hd, wbs element
static LEGACY_WBS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"S-(\d{7})-2-(\d{2})").unwrap());

/// SAP Wbs element for cost association
#[derive(Debug)]
pub enum Wbs {
    /// Hard Dollar WBS element
    Hd {
        /// Project number
        project: u32,
        /// Hard Dollar line ID
        id: u32
    },
    /// Legacy SAP WBS element
    Legacy {
        /// Project number
        project: u32,
        /// Shipment number
        shipment: u32
    }
}

impl TryFrom<&str> for Wbs {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if let Some(caps) = HD_WBS.captures(value) {
            let caps: [&str; 2] = caps.extract().1;

            // unwrap is safe here because the regex will assure that parse() does not fail
            return Ok(Self::Hd { project: caps[0].parse().unwrap(), id: caps[1].parse().unwrap() });
        }

        if let Some(caps) = LEGACY_WBS.captures(value) {
            let caps: [&str; 2] = caps.extract().1;

            // unwrap is safe here because the regex will assure that parse() does not fail
            return Ok(Self::Legacy { project: caps[0].parse().unwrap(), shipment: caps[1].parse().unwrap() });
        } 

        Err(format!("WBS element `{}` does not match either of the expected patterns `D-#######-#####` or `S-#######-2-##`", value))
    }
}

impl ToString for Wbs {
    fn to_string(&self) -> String {
        match self {
            Wbs::Hd { project, id } => format!("D-{project}-{id}"),
            Wbs::Legacy { project, shipment } => format!("S-{project}-2-{shipment:02}"),
        }
    }
}
