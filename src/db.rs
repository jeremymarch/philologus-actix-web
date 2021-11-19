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

use actix_web::{Error as AWError};

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

pub async fn get_def_by_word(pool: &SqlitePool, table:&str, word:&str) -> Result<Option<DefRow>, AWError> {
    let decoded_word = percent_decode_str(word).decode_utf8().unwrap();
    let query = format!("{}{}{}{}{}", "SELECT word,sortword,def,seq FROM ", table, " WHERE word = '", decoded_word, "' LIMIT 1;");

    let rec = sqlx::query_as::<_, DefRow>(&query)
    .fetch_one(&*pool)
    .await;
    /*
    match rec {
        Ok(rec) => {
        let res = match rec {
            None => return Ok(None),
            Some(rec) => {
                return Ok(Some(DefRow {
                    word: rec.unwrap().get(0),
                    sortword: rec.unwrap().get(1),
                    def: rec.unwrap().get(2),
                    seq: rec.unwrap().get(3)
                }))
            }
        }}
        Err() => return Err()
    }*/
    Ok(Some(rec.unwrap()))
}

pub async fn get_def_by_seq(pool: &SqlitePool, table:&str, id:u32) -> Result<Option<DefRow>, AWError> {
    let query = format!("{}{}{}{}{}", "SELECT word,sortword,def,seq FROM ", table, " WHERE seq = ", id, " LIMIT 1;");

    let rec = sqlx::query_as::<_, DefRow>(&query)
    .fetch_one(&*pool)
    .await;

    /*match rec {
        Ok(rec) => {
        let res = match rec {
            None => return Ok(None),
            Some(rec) => {
                return Ok(Some(DefRow {
                    word: rec.unwrap().get(0),
                    sortword: rec.unwrap().get(1),
                    def: rec.unwrap().get(2),
                    seq: rec.unwrap().get(3)
                }))
            }
        },
        Err() => return Err()
    }*/
    Ok(Some(rec.unwrap()))
}

//, SEQ_COL, $table, UNACCENTED_COL, $word, STATUS_COL, UNACCENTED_COL);
pub async fn get_seq_by_prefix(pool: &SqlitePool, table:&str, word:&str) -> Result<u32, AWError> {
    let query = format!("{}{}{}{}{}", "SELECT seq,word,def,sortword FROM ", table, " WHERE sortword >= '", word, "' ORDER BY sortword LIMIT 1;");
    
    let rec = sqlx::query_as::<_, DefRow>(&query)
    .fetch_one(&*pool)
    .await;

    Ok(rec.unwrap().seq)
    /*
    match rec {
        Ok(rec) => return rec,
        Err() 
    }
    
    match res {
        Ok(res) => return res.map_err(|_| AWError::from),
        Err(_) => {
            let query = format!("{}{}{}", "SELECT MAX(seq) FROM ", table, " LIMIT 1;");
            let rec = sqlx::query_as::<_, u32>(&query)
            .fetch_optional(&*pool)
            .await.map_err(|_|AWError::from);
        
            rec.map(|rec| rec.get(0) ).map_err(|_|AWError::from)
        }
    }*/
}

//, SEQ_COL, $table, UNACCENTED_COL, $word, STATUS_COL, UNACCENTED_COL);
pub async fn get_seq_by_word(pool: &SqlitePool, table:&str, word:&str) -> Result<u32, AWError> {
    let decoded_word = percent_decode_str(word).decode_utf8().unwrap();
    let query = format!("{}{}{}{}{}", "SELECT seq,word,def,sortword FROM ", table, " WHERE word = '", decoded_word, "' LIMIT 1;");

    let rec = sqlx::query_as::<_, DefRow>(&query)
    .fetch_one(&*pool)
    .await;

    Ok(rec.unwrap().seq)
}

//, ID_COL, WORD_COL, $table, $tagJoin, SEQ_COL, $middleSeq, STATUS_COL, $tagwhere, SEQ_COL, $req->limit * $req->page * -1, $req->limit);
pub async fn get_before(pool: &SqlitePool, table:&str, seq: u32, page: i32, limit: u32) -> Result<Vec<QueryResults>, AWError> {
    let query = format!("{}{}{}{}{}{}{}{}{}", "SELECT seq,word FROM ", table, " WHERE seq < ", seq, " ORDER BY seq DESC LIMIT ", page * limit as i32 * -1, ",", limit, ";");
        let res: Result<Vec<QueryResults>, sqlx::Error> = sqlx::query(&query)
        .map(|rec: SqliteRow| QueryResults {
            i: rec.get("seq"),
            r: (rec.get("word"), rec.get("seq"), 0)
        })
        .fetch_all(pool)
        .await;

        Ok(res.unwrap()) //fake for now
        /*
        match res {
            Ok(res) => return Ok(res),
            _ => return Err(res).map_err(AWError::from)
        };*/
}

//, ID_COL, WORD_COL, $table, $tagJoin, SEQ_COL, $middleSeq, STATUS_COL, $tagwhere, SEQ_COL, $req->limit * $req->page, $req->limit);
pub async fn get_equal_and_after(pool: &SqlitePool, table:&str, seq: u32, page: i32, limit: u32) -> Result<Vec<QueryResults>, AWError> {
    let query = format!("{}{}{}{}{}{}{}{}{}", "SELECT seq,word FROM ", table, " WHERE seq >= ", seq, " ORDER BY seq LIMIT ", page * limit as i32, ",", limit, ";");
    let res: Result<Vec<QueryResults>, sqlx::Error> = sqlx::query(&query)
    .map(|rec: SqliteRow| QueryResults {
        i: rec.get("seq"),
        r: (rec.get("word"), rec.get("seq"), 0)
    })
    .fetch_all(pool)
    .await;

    Ok(res.unwrap()) //fake for now
    /*
    match res {
        Ok(res) => return Ok(res),
        _ => return Err(res).map_err(AWError::from)
    };*/
}

/*
impl ResponseError for Error {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).finish()
    }
}
*/