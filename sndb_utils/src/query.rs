
use regex::Regex;
use std::sync::LazyLock;

use sysinteg_db::DbClient;

static PROGRAM: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\d{5,}(?:[\-_][[:alnum:]]+)*").unwrap());
static PART: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\d{3}?\d{4}[[:alpha:]]-([[:alnum:]]+(?:-[[:alnum:]])*)").unwrap());
static SHEET: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[SXW]\d{5}(?:[\-_][[:alnum:]]+)*").unwrap());
static STOCK_MATERIAL: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?:9-)?(?:HPS)?50W?(?:[TF][123])?-\d{4}[[:alpha:]]*").unwrap());
static PROJECT_MATERIAL: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\d{7}[[:alpha:]]\d{2}-\d{5}[[:alpha:]]*").unwrap());

pub enum Query {
    ProgramStatus(String),
    PartStatus(String),
    SheetStatus(String),
    MaterialStatus(String)
}

impl Query {
    pub async fn execute<'a>(&'a self, client: &'a mut DbClient) -> tiberius::Result<tiberius::QueryStream<'_>> {
        match self {
            Self::ProgramStatus(program) => client.query("EXEC GetProgramStatus @ProgramName=@P1", &[program]).await,
            Self::PartStatus(part) => client.query("EXEC GetPartStatus @ProgramName=@P1", &[part]).await,
            Self::SheetStatus(sheet) => client.query("EXEC GetSheetStatus @ProgramName=@P1", &[sheet]).await,
            Self::MaterialStatus(mm) => client.query("EXEC GetMaterialStatus @ProgramName=@P1", &[mm]).await,
        }
    }
}

impl TryFrom<&str> for Query {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            _ if PROGRAM.is_match(value) => Ok(Self::ProgramStatus(value.to_string())),
            _ if PART.is_match(value) => Ok(Self::PartStatus(value.to_string())),
            _ if SHEET.is_match(value) => Ok(Self::SheetStatus(value.to_string())),
            _ if STOCK_MATERIAL.is_match(value) => Ok(Self::MaterialStatus(value.to_string())),
            _ if PROJECT_MATERIAL.is_match(value) => Ok(Self::MaterialStatus(value.to_string())),
            _ => Err(format!("No query pattern matched for value `{}`", value))
        }
    }
}