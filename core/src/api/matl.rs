
//! Raw material types

use super::Wbs;

// TODO: should sigmanest have its own api?

/// Sigmanest sheet
#[derive(Debug, Default)]
pub struct Sheet {
    /// Sheet name (id)
    pub name: String,
    /// SAP Material Master
    pub mm: String,
    /// Heat number
    pub heat: String,
    /// Purchase Order number
    pub po: String,
    /// SAP WBS element
    pub wbs: Option<Wbs>
}
