/*
philologus-actix-web: philologus is a collection of digitized Greek and Latin Lexica

Copyright (C) 2021  Jeremy March

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>. 
*/

use actix_web::{HttpResponse, ResponseError, http::StatusCode};
use thiserror::Error;

use sqlx::sqlite::SqliteRow;
use sqlx::{FromRow, Row, SqlitePool };

use serde::{Deserialize, Serialize};
use percent_encoding::percent_decode_str;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PhilologusWords {
    GreekDefs { seq: i32, def: String },
}

//[{"i":1,"r":["Α α",1,0]},
// {"i":2,"r":["ἀ-",2,0]},
#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct QueryResults { 
    pub i: i32, 
    pub r: (String,u32,u32)
}

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct DefRow {
    pub word: String,
    pub sortword: String,
    pub def: String,
    pub seq: u32
}

pub async fn get_def_by_word(pool: &SqlitePool, table:&str, word:&str) -> Result<Option<DefRow>, PhilologusError> {
    let decoded_word = percent_decode_str(word).decode_utf8().map_err(map_utf8_error)?;
    let query = format!("SELECT word,sortword,def,seq FROM {} WHERE word = '{}' LIMIT 1;", table, decoded_word);

    let rec = sqlx::query_as::<_, DefRow>(&query)
    .fetch_one(&*pool)
    .await.map_err(map_sqlx_error)?;

    Ok(Some(rec))
}

pub async fn get_def_by_seq(pool: &SqlitePool, table:&str, id:u32) -> Result<Option<DefRow>, PhilologusError> {
    let query = format!("SELECT word,sortword,def,seq FROM {} WHERE seq = {} LIMIT 1;", table, id);

    let rec = sqlx::query_as::<_, DefRow>(&query)
    .fetch_one(&*pool)
    .await.map_err(map_sqlx_error)?;

    Ok(Some(rec))
}

pub async fn get_seq_by_prefix(pool: &SqlitePool, table:&str, word:&str) -> Result<u32, PhilologusError> {
    let query = format!("SELECT seq,word,def,sortword FROM {} WHERE sortword >= '{}' ORDER BY sortword LIMIT 1;", table, word);
    
    let rec = sqlx::query_as::<_, DefRow>(&query)
    .fetch_one(&*pool)
    .await;

    match rec {
        Ok(r) => Ok(r.seq),
        Err(sqlx::Error::RowNotFound) => {
            let query = format!("SELECT MAX(seq) as seq,word,def,sortword FROM {} LIMIT 1;", table);
            let rec = sqlx::query_as::<_, DefRow>(&query)  //fake it by loading it into DefRow for now
            .fetch_one(&*pool)
            .await.map_err(map_sqlx_error)?;
        
            Ok(rec.seq)
        }
        _ => Err(PhilologusError::Unknown)
    }
}

pub async fn get_seq_by_word(pool: &SqlitePool, table:&str, word:&str) -> Result<u32, PhilologusError> {
    let decoded_word = percent_decode_str(word).decode_utf8().map_err(map_utf8_error)?;
    let query = format!("SELECT seq,word,def,sortword FROM {} WHERE word = '{}' LIMIT 1;", table, decoded_word);

    let rec = sqlx::query_as::<_, DefRow>(&query)
    .fetch_one(&*pool)
    .await.map_err(map_sqlx_error)?;

    Ok(rec.seq)
}

pub async fn get_before(pool: &SqlitePool, table:&str, seq: u32, page: i32, limit: u32) -> Result<Vec<QueryResults>, PhilologusError> {
    let query = format!("SELECT seq,word FROM {} WHERE seq < {} ORDER BY seq DESC LIMIT {},{};", table, seq, page * limit as i32 * -1, limit);
    let res: Result<Vec<QueryResults>, sqlx::Error> = sqlx::query(&query)
    .map(|rec: SqliteRow| QueryResults {
        i: rec.get("seq"),
        r: (rec.get("word"), rec.get("seq"), 0)
    })
    .fetch_all(pool)
    .await;

    res.map_err(map_sqlx_error)
}

pub async fn get_equal_and_after(pool: &SqlitePool, table:&str, seq: u32, page: i32, limit: u32) -> Result<Vec<QueryResults>, PhilologusError> {
    let query = format!("SELECT seq,word FROM {} WHERE seq >= {} ORDER BY seq LIMIT {},{};", table, seq, page * limit as i32, limit);
    let res: Result<Vec<QueryResults>, sqlx::Error> = sqlx::query(&query)
    .map(|rec: SqliteRow| QueryResults {
        i: rec.get("seq"),
        r: (rec.get("word"), rec.get("seq"), 0)
    })
    .fetch_all(pool)
    .await;

    res.map_err(map_sqlx_error)
}

#[derive(Error, Debug)]
pub enum PhilologusError {
    /*#[error("Requested file was not found")]
    NotFound,
    #[error("You are forbidden to access requested file.")]
    Forbidden,*/
    #[error("Unknown Internal Error")]
    Unknown
}
/*
impl From<sqlx::Error> for PhilologusError {
    fn from(err: sqlx::Error) -> PhilologusError {
        PhilologusError {
            message: Some(err.to_string()),
            err_type: CustomErrorType::DieselError,
        }
    }
}
*/
impl PhilologusError {
    pub fn name(&self) -> String {
        match self {
            /*Self::NotFound => "NotFound".to_string(),
            Self::Forbidden => "Forbidden".to_string(),*/
            Self::Unknown => "Unknown".to_string(),
        }
    }
}
impl ResponseError for PhilologusError {
    fn status_code(&self) -> StatusCode {
        match *self {
            /*Self::NotFound  => StatusCode::NOT_FOUND,
            Self::Forbidden => StatusCode::FORBIDDEN,*/
            Self::Unknown => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status_code = self.status_code();
        let error_response = ErrorResponse {
            code: status_code.as_u16(),
            message: self.to_string(),
            error: self.name(),
        };
        HttpResponse::build(status_code).json(error_response)
    }
}

fn map_sqlx_error(e: sqlx::Error) -> PhilologusError {   
    match e {
        sqlx::Error::Configuration { .. } => PhilologusError::Unknown,
        sqlx::Error::Database { .. } => PhilologusError::Unknown,
        sqlx::Error::Io { .. } => PhilologusError::Unknown,
        sqlx::Error::Tls { .. } => PhilologusError::Unknown,
        sqlx::Error::Protocol { .. } => PhilologusError::Unknown,
        sqlx::Error::RowNotFound => PhilologusError::Unknown,
        sqlx::Error::TypeNotFound { .. } => PhilologusError::Unknown,
        sqlx::Error::ColumnIndexOutOfBounds { .. } => PhilologusError::Unknown,
        sqlx::Error::ColumnNotFound { .. } => PhilologusError::Unknown,
        sqlx::Error::ColumnDecode { .. } => PhilologusError::Unknown,
        sqlx::Error::Decode { .. } => PhilologusError::Unknown,
        sqlx::Error::PoolTimedOut => PhilologusError::Unknown,
        sqlx::Error::PoolClosed => PhilologusError::Unknown,
        sqlx::Error::WorkerCrashed => PhilologusError::Unknown,
        sqlx::Error::Migrate { .. } => PhilologusError::Unknown,
        _ => PhilologusError::Unknown,
    }
}

fn map_utf8_error(_e: std::str::Utf8Error) -> PhilologusError {   
    PhilologusError::Unknown
}

#[derive(Serialize)]
struct ErrorResponse {
    code: u16,
    error: String,
    message: String,
}
