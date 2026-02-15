use crate::generate_sql;
use thiserror::Error;
use crate::generate_sql::{CompressionType, DataType};

#[derive(Debug, Error)]
pub enum SqlGeneratorError {
    #[error("Invalid level")]
    InvalidLevel
}

fn generate_drs(level: i32) -> Result<(i32, i32), SqlGeneratorError> {
    match level {
        0 => Ok((300, 1)),
        1 => Ok((3600, 12)),
        2 => Ok((86400, 288)),
        3 => Ok((864000, 2880)),
        4 => Ok((8640000, 28800)),
        5 => Ok((31536000, 105120)),
        _ => Err(SqlGeneratorError::InvalidLevel)
    }
}

pub fn gl_server_stat(table: &str, level: i32) -> Result<String, SqlGeneratorError> {
    let (ranges, bs) = generate_drs(level)?;
    Ok(generate_sql::generate_server_stat(table, ranges.to_string().as_str(), bs.to_string().as_str()))
}

pub fn gl_mc_stat(table: &str, level: i32) -> Result<String, SqlGeneratorError> {
    let (ranges, bs) = generate_drs(level)?;
    Ok(generate_sql::generate_mc_stat(table, ranges.to_string().as_str(), bs.to_string().as_str()))
}

pub fn gl_current_general(compression_type: CompressionType, data_type: DataType, table: &str, level: i32) -> Result<String, SqlGeneratorError> {
    let (ranges, _) = generate_drs(level)?;
    Ok(generate_sql::generate_current_general(compression_type, data_type, table, ranges))
}
