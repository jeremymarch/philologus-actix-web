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

use sqlx::sqlite::SqliteRow;
use sqlx::{FromRow, Row, SqlitePool };
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PhilologusWords {
    GreekDefs { seq: u32, def: String },
}

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct DefRow {
    pub word: String,
    pub sortword: String,
    pub def: String,
    pub seq: u32
}

pub async fn insert_log(pool: &SqlitePool, accessed: u128, lex:u8, wordid:u32, ip:&str, agent:&str) -> Result<u32, sqlx::Error> {
    //let query = format!("INSERT INTO test VALUES (2)");
    let query = format!("INSERT INTO log VALUES (NULL, {}, {}, {}, '{}', '{}')", accessed, lex, wordid, ip, agent);
    sqlx::query(&query).execute(pool).await?;

    Ok(1)
}

pub async fn get_def_by_word(pool: &SqlitePool, table:&str, word:&str) -> Result<DefRow, sqlx::Error> {
    let query = format!("SELECT word,sortword,def,seq FROM {} WHERE word = '{}' LIMIT 1;", table, word);

    let rec = sqlx::query_as::<_, DefRow>(&query)
    .fetch_one(pool)
    .await?;

    Ok(rec)
}

pub async fn get_def_by_seq(pool: &SqlitePool, table:&str, id:u32) -> Result<DefRow, sqlx::Error> {
    let query = format!("SELECT word,sortword,def,seq FROM {} WHERE seq = {} LIMIT 1;", table, id);

    let rec = sqlx::query_as::<_, DefRow>(&query)
    .fetch_one(pool)
    .await?;

    Ok(rec)
}

pub async fn get_seq_by_prefix(pool: &SqlitePool, table:&str, prefix:&str) -> Result<u32, sqlx::Error> {
    let query = format!("SELECT seq,word,def,sortword FROM {} WHERE sortword >= '{}' ORDER BY sortword LIMIT 1;", table, prefix);
    
    let rec = sqlx::query_as::<_, DefRow>(&query)
    .fetch_one(pool)
    .await;

    match rec {
        Ok(r) => Ok(r.seq),
        Err(sqlx::Error::RowNotFound) => { //not found, return seq of last word
            let max_query = format!("SELECT MAX(seq) as seq,word,def,sortword FROM {} LIMIT 1;", table);
            let max_rec = sqlx::query_as::<_, DefRow>(&max_query)  //fake it by loading it into DefRow for now
            .fetch_one(pool)
            .await?;
        
            Ok(max_rec.seq)
        },
        Err(r) => Err(r)
    }
}

pub async fn get_seq_by_word(pool: &SqlitePool, table:&str, word:&str) -> Result<u32, sqlx::Error> {
    let query = format!("SELECT seq,word,def,sortword FROM {} WHERE word = '{}' LIMIT 1;", table, word);

    let rec = sqlx::query_as::<_, DefRow>(&query)
    .fetch_one(pool)
    .await?;

    Ok(rec.seq)
}

pub async fn get_before(pool: &SqlitePool, table:&str, seq: u32, page: i32, limit: u32) -> Result<Vec<(String,u32)>, sqlx::Error> {
    let query = format!("SELECT seq,word FROM {} WHERE seq < {} ORDER BY seq DESC LIMIT {},{};", table, seq, -page * limit as i32, limit);
    let res: Result<Vec<(String,u32)>, sqlx::Error> = sqlx::query(&query)
    .map(|rec: SqliteRow| (rec.get("word"),rec.get("seq")) )
    .fetch_all(pool)
    .await;

    res
}

pub async fn get_equal_and_after(pool: &SqlitePool, table:&str, seq: u32, page: i32, limit: u32) -> Result<Vec<(String,u32)>, sqlx::Error> {
    let query = format!("SELECT seq,word FROM {} WHERE seq >= {} ORDER BY seq LIMIT {},{};", table, seq, page * limit as i32, limit);
    let res: Result<Vec<(String,u32)>, sqlx::Error> = sqlx::query(&query)
    .map(|rec: SqliteRow| (rec.get("word"),rec.get("seq")) )
    .fetch_all(pool)
    .await;

    res
}
