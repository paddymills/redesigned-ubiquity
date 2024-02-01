
//! Database utilities

/// convert SQL Server column to string, regardless of datatype
pub fn column_to_str(column: ColumnData) -> String {
    use ColumnData::*;
    match column {
        U8(n)  => n.unwrap().to_string(),
        I16(n) => n.unwrap().to_string(),
        I32(n) => n.unwrap().to_string(),
        I64(n) => n.unwrap().to_string(),
        F32(n) => n.unwrap().to_string(),
        F64(n) => n.unwrap().to_string(),
        Numeric(n) => n.unwrap().to_string(),

        String(s) => s.unwrap().to_string(),

        _ => unimplemented!()
    }
}

/// Converts a SQL row to a tab-delimited string
pub fn row_to_string(row: Row) -> String {
    row.into_iter()
        .map(column_to_str)

        // generally speaking, one the these lines should be no more than 128 characters
        //  so we set an initial capacity of 128 to try to avoid reallocations
        .fold(String::with_capacity(128), |acc, s| acc + &s + "\t")
        
        .trim_end() // remove trailing '\t'
        .into()
}
