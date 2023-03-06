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
use tracing::info;

use crate::SynopsisSaverRequest;
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqliteRow;
use sqlx::{FromRow, Row, SqlitePool};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PhilologusWords {
    GreekDefs { seq: u32, def: String },
}

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct DefRow {
    pub word: String,
    pub sortword: String,
    pub def: String,
    pub seq: u32,
}

pub async fn get_synopsis_list(
    pool: &SqlitePool,
) -> Result<Vec<(i64, i64, String, String, String)>, sqlx::Error> {
    let query =
        "SELECT id, updated, sname, advisor, selectedverb FROM synopsisresults ORDER BY updated DESC;";
    let res: Vec<(i64, i64, String, String, String)> =
        sqlx::query_as(query).fetch_all(pool).await?;

    Ok(res)
}

pub async fn get_synopsis_result(
    pool: &SqlitePool,
    id: u32,
) -> Result<Vec<(i64, i64, String, String, String)>, sqlx::Error> {
    let query = format!("SELECT id, updated, sname, advisor, selectedverb FROM synopsisresults WHERE id={} ORDER BY updated DESC;", id);
    let res: Vec<(i64, i64, String, String, String)> =
        sqlx::query_as(&query).fetch_all(pool).await?;

    Ok(res)
}

pub async fn insert_synopsis(
    pool: &SqlitePool,
    info: &SynopsisSaverRequest,
    accessed: u128,
    ip: &str,
    agent: &str,
) -> Result<u32, sqlx::Error> {
    let query = format!("INSERT INTO synopsisresults VALUES (NULL, {}, '{}', '{}', {}, '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}')", 
        accessed, info.sname, info.advisor, info.unit, info.verb, info.pp,
        info.number, info.person, info.ptcgender, info.ptcnumber, info.ptccase, ip, agent,
        info.r.join("', '"));
    sqlx::query(&query).execute(pool).await?;

    Ok(1)
}

pub async fn insert_log(
    pool: &SqlitePool,
    accessed: u128,
    lex: u8,
    wordid: u32,
    ip: &str,
    agent: &str,
) -> Result<u32, sqlx::Error> {
    let query = format!(
        "INSERT INTO log VALUES (NULL, {}, {}, {}, '{}', '{}');",
        accessed, lex, wordid, ip, agent
    );
    sqlx::query(&query).execute(pool).await?;

    Ok(1)
}

pub async fn get_def_by_word(
    pool: &SqlitePool,
    table: &str,
    word: &str,
) -> Result<DefRow, sqlx::Error> {
    info!(table, word, "get_def_by_word()");

    let query =
        "SELECT word, sortword, def, seq FROM words WHERE word = $1 AND lexicon = $2 LIMIT 1;";
    let rec = sqlx::query_as::<_, DefRow>(query)
        .bind(word)
        .bind(table)
        .fetch_one(pool)
        .await?;

    Ok(rec)
}

pub async fn get_def_by_seq(pool: &SqlitePool, id: u32) -> Result<DefRow, sqlx::Error> {
    info!(id, "get_def_by_seq()");

    let query = "SELECT word, sortword, def, seq FROM words WHERE seq = $1 LIMIT 1;";

    let rec = sqlx::query_as::<_, DefRow>(query)
        .bind(id)
        .fetch_one(pool)
        .await?;

    Ok(rec)
}

pub async fn get_seq_by_prefix(
    pool: &SqlitePool,
    table: &str,
    prefix: &str,
) -> Result<u32, sqlx::Error> {
    info!(table, prefix, "get_seq_by_prefix()");

    let query = "SELECT seq, word, def, sortword FROM words WHERE sortword >= $1 AND lexicon = $2 ORDER BY sortword LIMIT 1;";

    let rec = sqlx::query_as::<_, DefRow>(query)
        .bind(prefix)
        .bind(table)
        .fetch_one(pool)
        .await;

    match rec {
        Ok(r) => Ok(r.seq),
        Err(sqlx::Error::RowNotFound) => {
            //not found, return seq of last word
            let max_query = "SELECT MAX(seq) as seq, word, def, sortword FROM words WHERE lexicon = $1 LIMIT 1;";
            let max_rec = sqlx::query_as::<_, DefRow>(max_query) //fake it by loading it into DefRow for now
                .bind(table)
                .fetch_one(pool)
                .await?;

            Ok(max_rec.seq)
        }
        Err(r) => Err(r),
    }
}

pub async fn get_seq_by_word(
    pool: &SqlitePool,
    table: &str,
    word: &str,
) -> Result<u32, sqlx::Error> {
    info!(table, word, "get_seq_by_word()");

    let query =
        "SELECT seq, word, def, sortword FROM words WHERE word = $1 AND lexicon = $2 LIMIT 1;";

    let rec = sqlx::query_as::<_, DefRow>(query)
        .bind(word)
        .bind(table)
        .fetch_one(pool)
        .await?;

    Ok(rec.seq)
}

pub async fn get_before(
    pool: &SqlitePool,
    table: &str,
    seq: u32,
    page: i32,
    limit: u32,
) -> Result<Vec<(u32, String,)>, sqlx::Error> {
    info!(seq, table, page, limit, "get_before()");

    let query = "SELECT seq, word FROM words WHERE seq < $1 AND lexicon = $2 ORDER BY seq DESC LIMIT $3, $4;";
    let res: Result<Vec<(u32, String,)>, sqlx::Error> = sqlx::query(query)
        .bind(seq)
        .bind(table)
        .bind(-page * limit as i32)
        .bind(limit)
        .map(|rec: SqliteRow| (rec.get("seq"), rec.get("word"),))
        .fetch_all(pool)
        .await;

    res
}

pub async fn get_equal_and_after(
    pool: &SqlitePool,
    table: &str,
    seq: u32,
    page: i32,
    limit: u32,
) -> Result<Vec<(u32, String,)>, sqlx::Error> {
    info!(seq, table, page, limit, "get_after()");

    let query =
        "SELECT seq, word FROM words WHERE seq >= $1 AND lexicon = $2 ORDER BY seq LIMIT $3, $4;";
    let res: Result<Vec<(u32, String,)>, sqlx::Error> = sqlx::query(query)
        .bind(seq)
        .bind(table)
        .bind(page * limit as i32)
        .bind(limit)
        .map(|rec: SqliteRow| (rec.get("seq"), rec.get("word"),))
        .fetch_all(pool)
        .await;

    res
}
