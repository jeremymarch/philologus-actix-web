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

use sqlx::{FromRow, AnyPool };
use sqlx::any::AnyRow;
use sqlx::Row;
use serde::{Deserialize, Serialize};
use crate::SynopsisSaverRequest;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PhilologusWords {
    GreekDefs { seq: i32, def: String },
}

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct DefRow {
    pub word: String,
    pub sortword: String,
    pub def: String,
    pub seq: i32
}

pub async fn get_synopsis_list(pool: &AnyPool) -> Result<Vec<(i64,i64,String,String,String)>, sqlx::Error> {
    let query = "SELECT id,updated,sname,advisor,selectedverb FROM synopsisresults ORDER BY updated DESC;";
    let res: Vec<(i64,i64,String,String,String)> = sqlx::query(query)
    .map(|rec: AnyRow| {
        (
            rec.get("id"),
            rec.get("updated"),
            rec.get("sname"),
            rec.get("advisor"),
            rec.get("selectedverb"),
        )
    })
    .fetch_all(pool)
    .await?;

    Ok(res)
}

pub async fn get_synopsis_result(pool: &AnyPool, id:i32) -> Result<Vec<(i64,i64,String,String,String)>, sqlx::Error> {
    let query = format!("SELECT id,updated,sname,advisor,selectedverb FROM synopsisresults WHERE id={} ORDER BY updated DESC;", id);
    let res: Vec<(i64,i64,String,String,String)> = sqlx::query(&query)
    .map(|rec: AnyRow| {
        (
            rec.get("id"),
            rec.get("updated"),
            rec.get("sname"),
            rec.get("advisor"),
            rec.get("selectedverb"),
        )
    })
    .fetch_all(pool)
    .await?;

    Ok(res)
}

pub async fn insert_synopsis(pool: &AnyPool, info:&SynopsisSaverRequest, accessed: u128, ip:&str, agent:&str) -> Result<i32, sqlx::Error> {
    let query = format!("INSERT INTO synopsisresults VALUES (NULL, {}, '{}', '{}', {}, '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}')", 
        accessed, info.sname, info.advisor, info.unit, info.verb, info.pp, 
        info.number, info.person, info.ptcgender, info.ptcnumber, info.ptccase, ip, agent, 
        info.r.join("', '"));
    sqlx::query(&query).execute(pool).await?;

    Ok(1)
}

pub async fn insert_log(pool: &AnyPool, accessed: u128, lex:i32, wordid:i32, ip:&str, agent:&str) -> Result<i32, sqlx::Error> {
    let query = format!("INSERT INTO log VALUES (NULL, {}, {}, {}, '{}', '{}')", accessed, lex, wordid, ip, agent);
    sqlx::query(&query).execute(pool).await?;

    Ok(1)
}

pub async fn get_def_by_word(pool: &AnyPool, table:&str, word:&str) -> Result<DefRow, sqlx::Error> {
    let query = format!("SELECT word,sortword,def,seq FROM {} WHERE word = ? LIMIT 1;", table);

    let rec = sqlx::query(&query)
    .bind(word)
    .map(|rec: AnyRow| {
        DefRow {
            word: rec.get("word"),
            sortword: rec.get("sortword"),
            def: rec.get("def"),
            seq: rec.get("seq"),
        }
    })
    .fetch_one(pool)
    .await?;

    Ok(rec)
}

pub async fn get_def_by_seq(pool: &AnyPool, table:&str, id:i32) -> Result<DefRow, sqlx::Error> {
    let query = format!("SELECT word,sortword,def,seq FROM {} WHERE seq = ? LIMIT 1;", table);
    
    let rec = sqlx::query(&query)
    .bind(id)
    .map(|rec: AnyRow| {
        DefRow {
            word: rec.get("word"),
            sortword: rec.get("sortword"),
            def: rec.get("def"),
            seq: rec.get("seq"),
        }
    })
    .fetch_one(pool)
    .await?;

    Ok(rec)
}

pub async fn get_seq_by_prefix(pool: &AnyPool, table:&str, prefix:&str) -> Result<i32, sqlx::Error> {
    let query = format!("SELECT seq,word,def,sortword FROM {} WHERE sortword >= ? ORDER BY sortword LIMIT 1;", table);

    let rec = sqlx::query(&query)
    .bind(prefix)
    .map(|rec: AnyRow| {
        DefRow {
            word: rec.get("word"),
            sortword: rec.get("sortword"),
            def: rec.get("def"),
            seq: rec.get("seq"),
        }
    })
    .fetch_one(pool)
    .await;

    match rec {
        Ok(r) => Ok(r.seq),
        Err(sqlx::Error::RowNotFound) => { //not found, return seq of last word
            let max_query = format!("SELECT MAX(seq) as seq,word,def,sortword FROM {} LIMIT 1;", table);
            let max_rec = sqlx::query(&max_query)  //fake it by loading it into DefRow for now
            .map(|rec: AnyRow| {
                DefRow {
                    word: rec.get("word"),
                    sortword: rec.get("sortword"),
                    def: rec.get("def"),
                    seq: rec.get("seq"),
                }
            })
            .fetch_one(pool)
            .await?;
        
            Ok(max_rec.seq)
        },
        Err(r) => Err(r)
    }
}

pub async fn get_seq_by_word(pool: &AnyPool, table:&str, word:&str) -> Result<i32, sqlx::Error> {
    let query = format!("SELECT seq,word,def,sortword FROM {} WHERE word = ? LIMIT 1;", table);
    
    let rec = sqlx::query(&query)
    .bind(word)
    .map(|rec: AnyRow| {
        DefRow {
            word: rec.get("word"),
            sortword: rec.get("sortword"),
            def: rec.get("def"),
            seq: rec.get("seq"),
        }
    })
    .fetch_one(pool)
    .await?;

    Ok(rec.seq)
}

pub async fn get_before(pool: &AnyPool, table:&str, seq: i32, page: i32, limit: i32) -> Result<Vec<(String,i32)>, sqlx::Error> {
    let query = format!("SELECT seq,word FROM {} WHERE seq < ? ORDER BY seq DESC LIMIT ?,?;", table);
    let res: Result<Vec<(String,i32)>, sqlx::Error> = sqlx::query(&query)
    .bind(seq)
    .bind(-page * limit as i32)
    .bind(limit)
    .map(|rec: AnyRow| (rec.get("word"),rec.get("seq")) )
    .fetch_all(pool)
    .await;

    res
}

pub async fn get_equal_and_after(pool: &AnyPool, table:&str, seq: i32, page: i32, limit: i32) -> Result<Vec<(String,i32)>, sqlx::Error> {
    let query = format!("SELECT seq,word FROM {} WHERE seq >= ? ORDER BY seq LIMIT ?,?;", table);
    let res: Result<Vec<(String,i32)>, sqlx::Error> = sqlx::query(&query)
    .bind(seq)
    .bind(page * limit as i32)
    .bind(limit)
    .map(|rec: AnyRow| (rec.get("word"),rec.get("seq")) )
    .fetch_all(pool)
    .await;

    res
}
