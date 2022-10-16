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
use sqlx::{FromRow, Row, AnyRow, SqlitePool, AnyPool };
use serde::{Deserialize, Serialize};
use crate::SynopsisSaverRequest;

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

pub async fn get_synopsis_list(pool: &AnyPool) -> Result<Vec<(i64,i64,String,String,String)>, sqlx::Error> {
    let query = "SELECT id,updated,sname,advisor,selectedverb FROM synopsisresults ORDER BY updated DESC;";
    let res: Vec<(i64,i64,String,String,String)> = sqlx::query_as(&query)
    .fetch_all(pool)
    .await?;

    Ok(res)
}

pub async fn get_synopsis_result(pool: &AnyPool, id:u32) -> Result<Vec<(i64,i64,String,String,String)>, sqlx::Error> {
    let query = format!("SELECT id,updated,sname,advisor,selectedverb FROM synopsisresults WHERE id={} ORDER BY updated DESC;", id);
    let res: Vec<(i64,i64,String,String,String)> = sqlx::query_as(&query)
    .fetch_all(pool)
    .await?;

    Ok(res)
}

pub async fn insert_synopsis(pool: &AnyPool, info:&SynopsisSaverRequest, accessed: u128, ip:&str, agent:&str) -> Result<u32, sqlx::Error> {
    let query = format!("INSERT INTO synopsisresults VALUES (NULL, {}, '{}', '{}', {}, '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}')", 
        accessed, info.sname, info.advisor, info.unit, info.verb, info.pp, 
        info.number, info.person, info.ptcgender, info.ptcnumber, info.ptccase, ip, agent, 
        info.r.join("', '"));
    sqlx::query(&query).execute(pool).await?;

    Ok(1)
}

pub async fn insert_log(pool: &AnyPool, accessed: u128, lex:u8, wordid:u32, ip:&str, agent:&str) -> Result<u32, sqlx::Error> {
    let query = format!("INSERT INTO log VALUES (NULL, {}, {}, {}, '{}', '{}')", accessed, lex, wordid, ip, agent);
    sqlx::query(&query).execute(pool).await?;

    Ok(1)
}

pub async fn get_def_by_word(pool: &AnyPool, table:&str, word:&str) -> Result<DefRow, sqlx::Error> {
    let query = format!("SELECT word,sortword,def,seq FROM {} WHERE word = ? LIMIT 1;", table);

    let rec = sqlx::query_as::<_, DefRow>(&query)
    .bind(word)
    .fetch_one(pool)
    .await?;

    Ok(rec)
}

pub async fn get_def_by_seq(pool: &AnyPool, table:&str, id:u32) -> Result<DefRow, sqlx::Error> {
    let query = format!("SELECT word,sortword,def,seq FROM {} WHERE seq = ? LIMIT 1;", table);

    let rec = sqlx::query_as::<_, DefRow>(&query)
    .bind(id)
    .fetch_one(pool)
    .await?;

    Ok(rec)
}

pub async fn get_seq_by_prefix(pool: &AnyPool, table:&str, prefix:&str) -> Result<u32, sqlx::Error> {
    let query = format!("SELECT seq,word,def,sortword FROM {} WHERE sortword >= ? ORDER BY sortword LIMIT 1;", table);
    
    let rec = sqlx::query_as::<_, DefRow>(&query)
    .bind(prefix)
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

pub async fn get_seq_by_word(pool: &AnyPool, table:&str, word:&str) -> Result<u32, sqlx::Error> {
    let query = format!("SELECT seq,word,def,sortword FROM {} WHERE word = ? LIMIT 1;", table);

    let rec = sqlx::query_as::<_, DefRow>(&query)
    .bind(word)
    .fetch_one(pool)
    .await?;

    Ok(rec.seq)
}

pub async fn get_before(pool: &AnyPool, table:&str, seq: u32, page: i32, limit: u32) -> Result<Vec<(String,u32)>, sqlx::Error> {
    let query = format!("SELECT seq,word FROM {} WHERE seq < ? ORDER BY seq DESC LIMIT ?,?;", table);
    let res: Result<Vec<(String,u32)>, sqlx::Error> = sqlx::query(&query)
    .bind(seq)
    .bind(-page * limit as i32)
    .bind(limit)
    .map(|rec: AnyRow| (rec.get("word"),rec.get("seq")) )
    .fetch_all(pool)
    .await;

    res
}

pub async fn get_equal_and_after(pool: &AnyPool, table:&str, seq: u32, page: i32, limit: u32) -> Result<Vec<(String,u32)>, sqlx::Error> {
    let query = format!("SELECT seq,word FROM {} WHERE seq >= ? ORDER BY seq LIMIT ?,?;", table);
    let res: Result<Vec<(String,u32)>, sqlx::Error> = sqlx::query(&query)
    .bind(seq)
    .bind(page * limit as i32)
    .bind(limit)
    .map(|rec: AnyRow| (rec.get("word"),rec.get("seq")) )
    .fetch_all(pool)
    .await;

    res
}
