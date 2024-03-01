
use chrono::NaiveDateTime;
use comfy_table::{Cell, Color, Row};
use sysinteg_core::api::{Sheet, Wbs};

pub const HEADER: [&str; 8] = ["Program", "Status", "Timestamp", "SAP MM", "Heat Number", "PO Number", "SheetName", "Operator"];
const DATE_FORMAT: &str = "%e.%b.%Y %k:%M %P";

#[derive(Debug)]
pub struct Program {
    pub name: String,
    pub state: ProgramState,
    pub sheet: Sheet,
}

impl Into<Row> for Program {
    fn into(self) -> Row {
        // "Program", "Status", "Timestamp", "SAP MM", "Heat Number", "PO Number", "SheetName", "Operator"
        let mut row = Row::new();
        row.add_cell(Cell::new(self.name));
        
        match &self.state {
            ProgramState::Active(timestamp) => {
                row
                    .add_cell(Cell::new("Active").fg(Color::Blue))
                    .add_cell(Cell::new(timestamp.format(DATE_FORMAT).to_string()))
                    .add_cell(Cell::new(self.sheet.mm))
                    .add_cell(Cell::new(""))
                    .add_cell(Cell::new(""))
                    .add_cell(Cell::new(self.sheet.name));
            },
            ProgramState::Deleted(timestamp) => {
                row
                    .add_cell(Cell::new("Deleted").fg(Color::Red))
                    .add_cell(Cell::new(timestamp.format(DATE_FORMAT).to_string()))
                    .add_cell(Cell::new(self.sheet.mm))
                    .add_cell(Cell::new(""))
                    .add_cell(Cell::new(""))
                    .add_cell(Cell::new(self.sheet.name));
            },
            ProgramState::Updated { timestamp, operator } => {
                row
                    .add_cell(Cell::new("Updated").fg(Color::Green))
                    .add_cell(Cell::new(timestamp.format(DATE_FORMAT).to_string()))
                    .add_cell(Cell::new(self.sheet.mm))
                    .add_cell(Cell::new(self.sheet.heat))
                    .add_cell(Cell::new(format!("{}", self.sheet.po)))
                    .add_cell(Cell::new(self.sheet.name));

                if let Some(operator) = operator {
                    row.add_cell(Cell::new(operator));
                }
            }
        }

        row
    }
}

impl From<&tiberius::Row> for Program {
    fn from(row: &tiberius::Row) -> Self {
        let state = ProgramState::from(row);

        match state {
            ProgramState::Updated { .. } => Self {
                name: row.get::<&str, _>("ProgramName").unwrap().into(),
                state,
                sheet: Sheet {
                    name: row.get::<&str, _>("SheetName").unwrap().into(),
                    mm: row.get::<&str, _>("MaterialMaster").unwrap_or_default().into(),
                    heat: row.get::<&str, _>("HeatNumber").unwrap_or_default().into(),
                    po: row.get::<&str, _>("PoNumber").unwrap_or_default().into(),
                    wbs: row.get::<&str, _>("Wbs").map(|wbs| Wbs::try_from(wbs).unwrap())
                }
            },
            _ => Self {
                name: row.get::<&str, _>("ProgramName").unwrap().into(),
                state,
                sheet: Sheet {
                    name: row.get::<&str, _>("SheetName").unwrap().into(),
                    mm: row.get::<&str, _>("MaterialMaster").unwrap_or_default().into(),
                    
                    ..Default::default()
                }
            }
        }
    }
}

/// State of a program
#[derive(Debug)]
pub enum ProgramState {
    /// Program is active
    Active (
        /// Date and time of last program posting
        chrono::NaiveDateTime,
    ),

    /// Program was deleted
    Deleted (
        /// Date and time of program deletion
        chrono::NaiveDateTime,
    ),

    /// Program updated
    Updated {
        /// program update time (heat swap time, not SimTrans)
        timestamp: chrono::NaiveDateTime,
        /// Heat swap operator
        operator: Option<String>,
    },
}

impl From<&tiberius::Row> for ProgramState {
    fn from(row: &tiberius::Row) -> Self {
        let timestamp: NaiveDateTime = row.get("Timestamp").unwrap();

        match row.get::<&str, _>("Status").unwrap() {
            "Active" => Self::Active(timestamp),
            "Deleted" => Self::Deleted(timestamp),
            "Updated" => Self::Updated { timestamp, operator: row.get::<&str, _>("Operator").map(Into::into) },
            unmatched => panic!("Received unexpected program status `{}`", unmatched)
        }
    }
}