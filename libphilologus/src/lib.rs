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
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqliteRow;
use sqlx::{FromRow, Row, SqlitePool};
use tracing::info;

// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub enum PhilologusWords {
//     GreekDefs { seq: u32, def: String },
// }
#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct LatinSynopsisResult {
    pub id: i64,
    pub updated: i64,
    pub sname: String,
    pub advisor: String,
    pub sgiday: i64,
    pub selectedverb: String,
    pub pp: String,
    pub verbnumber: String,
    pub verbperson: String,
    pub verbptcgender: String,
    pub verbptcnumber: String,
    pub verbptccase: String,
    pub ip: String,
    pub ua: String,
    pub status: i64,
    pub f0: String,
    pub f1: String,
    pub f2: String,
    pub f3: String,
    pub f4: String,
    pub f5: String,
    pub f6: String,
    pub f7: String,
    pub f8: String,
    pub f9: String,
    pub f10: String,
    pub f11: String,
    pub f12: String,
    pub f13: String,
    pub f14: String,
    pub f15: String,
    pub f16: String,
    pub f17: String,
    pub f18: String,
    pub f19: String,
    pub f20: String,
    pub f21: String,
    pub f22: String,
    pub f23: String,
    pub f24: String,
    pub f25: String,
    pub f26: String,
    pub f27: String,
    pub f28: String,
    pub f29: String,
    pub f30: String,
    pub f31: String,
    pub f32: String,
    pub f33: String,
    pub f34: String,
    pub f35: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct GreekSynopsisResult {
    pub id: i64,
    pub updated: i64,
    pub sname: String,
    pub advisor: String,
    pub sgiday: i64,
    pub selectedverb: String,
    pub pp: String,
    pub verbnumber: String,
    pub verbperson: String,
    pub verbptcgender: String,
    pub verbptcnumber: String,
    pub verbptccase: String,
    pub ip: String,
    pub ua: String,
    pub status: i64,
    pub f0: String,
    pub f1: String,
    pub f2: String,
    pub f3: String,
    pub f4: String,
    pub f5: String,
    pub f6: String,
    pub f7: String,
    pub f8: String,
    pub f9: String,
    pub f10: String,
    pub f11: String,
    pub f12: String,
    pub f13: String,
    pub f14: String,
    pub f15: String,
    pub f16: String,
    pub f17: String,
    pub f18: String,
    pub f19: String,
    pub f20: String,
    pub f21: String,
    pub f22: String,
    pub f23: String,
    pub f24: String,
    pub f25: String,
    pub f26: String,
    pub f27: String,
    pub f28: String,
    pub f29: String,
    pub f30: String,
    pub f31: String,
    pub f32: String,
    pub f33: String,
    pub f34: String,
    pub f35: String,
    pub f36: String,
    pub f37: String,
    pub f38: String,
    pub f39: String,
    pub f40: String,
    pub f41: String,
    pub f42: String,
    pub f43: String,
    pub f44: String,
    pub f45: String,
    pub f46: String,
    pub f47: String,
    pub f48: String,
    pub f49: String,
    pub f50: String,
    pub f51: String,
    pub f52: String,
    pub f53: String,
    pub f54: String,
    pub f55: String,
    pub f56: String,
    pub f57: String,
    pub f58: String,
    pub f59: String,
    pub f60: String,
    pub f61: String,
    pub f62: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SynopsisSaverRequest {
    pub advisor: String,
    pub unit: usize,
    pub sname: String,
    pub number: usize,
    pub person: usize,
    pub pp: String,
    pub ptccase: Option<usize>,
    pub ptcgender: Option<usize>,
    pub ptcnumber: Option<usize>,
    pub r: Vec<String>,
    pub verb: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct DefRow {
    pub word: String,
    pub sortword: String,
    pub def: String,
    pub seq: u32,
}

pub async fn greek_get_synopsis_list(
    pool: &SqlitePool,
) -> Result<Vec<(i64, i64, String, String, String)>, sqlx::Error> {
    let query = "SELECT id, updated, sname, advisor, selectedverb FROM greeksynopsisresults ORDER BY updated DESC;";
    let res: Vec<(i64, i64, String, String, String)> =
        sqlx::query_as(query).fetch_all(pool).await?;

    Ok(res)
}

pub async fn greek_get_synopsis_result(
    pool: &SqlitePool,
    id: u32,
) -> Result<Vec<GreekSynopsisResult>, sqlx::Error> {
    let query = format!(
        "SELECT * FROM greeksynopsisresults WHERE id={} ORDER BY updated DESC;",
        id
    );
    let res: Vec<GreekSynopsisResult> = sqlx::query_as::<_, GreekSynopsisResult>(&query)
        .fetch_all(pool)
        .await?;

    Ok(res)
}

pub async fn greek_insert_synopsis(
    pool: &SqlitePool,
    info: &SynopsisSaverRequest,
    accessed: u128,
    ip: &str,
    agent: &str,
) -> Result<u32, sqlx::Error> {
    let query = format!(
        "INSERT INTO greeksynopsisresults VALUES (NULL, {}, '{}', '{}', {}, '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}', {}, '{}')",
        accessed,
        info.sname,
        info.advisor,
        info.unit,
        info.verb,
        info.pp,
        info.number,
        info.person,
        info.ptcgender.unwrap_or(999),
        info.ptcnumber.unwrap_or(999),
        info.ptccase.unwrap_or(999),
        ip,
        agent,
        1,
        info.r.join("', '")
    );
    sqlx::query(&query).execute(pool).await?;

    Ok(1)
}

pub async fn latin_get_synopsis_list(
    pool: &SqlitePool,
) -> Result<Vec<(i64, i64, String, String, String)>, sqlx::Error> {
    let query = "SELECT id, updated, sname, advisor, selectedverb FROM latinsynopsisresults ORDER BY updated DESC;";
    let res: Vec<(i64, i64, String, String, String)> =
        sqlx::query_as(query).fetch_all(pool).await?;

    Ok(res)
}

pub async fn latin_get_synopsis_result(
    pool: &SqlitePool,
    id: u32,
) -> Result<Vec<LatinSynopsisResult>, sqlx::Error> {
    let query = format!(
        "SELECT * FROM latinsynopsisresults WHERE id={} ORDER BY updated DESC;",
        id
    );
    let res: Vec<LatinSynopsisResult> = sqlx::query_as::<_, LatinSynopsisResult>(&query)
        .fetch_all(pool)
        .await?;

    Ok(res)
}

// let rec = sqlx::query_as::<_, DefRow>(query)
//         .bind(word)
//         .bind(table)
//         .fetch_one(pool)
//         .await?;

pub async fn latin_insert_synopsis(
    pool: &SqlitePool,
    info: &SynopsisSaverRequest,
    accessed: u128,
    ip: &str,
    agent: &str,
) -> Result<u32, sqlx::Error> {
    let query = format!(
        "INSERT INTO latinsynopsisresults VALUES (NULL, {}, '{}', '{}', {}, '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}', {}, '{}')",
        accessed,
        info.sname,
        info.advisor,
        info.unit,
        info.verb,
        info.pp,
        info.number,
        info.person,
        info.ptcgender.unwrap_or(999),
        info.ptcnumber.unwrap_or(999),
        info.ptccase.unwrap_or(999),
        ip,
        agent,
        1,
        info.r.join("', '")
    );
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
) -> Result<Vec<(u32, String)>, sqlx::Error> {
    info!(seq, table, page, limit, "get_before()");

    let query = "SELECT seq, word FROM words WHERE seq < $1 AND lexicon = $2 ORDER BY seq DESC LIMIT $3, $4;";
    let res: Result<Vec<(u32, String)>, sqlx::Error> = sqlx::query(query)
        .bind(seq)
        .bind(table)
        .bind(-page * limit as i32)
        .bind(limit)
        .map(|rec: SqliteRow| (rec.get("seq"), rec.get("word")))
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
) -> Result<Vec<(u32, String)>, sqlx::Error> {
    info!(seq, table, page, limit, "get_after()");

    let query =
        "SELECT seq, word FROM words WHERE seq >= $1 AND lexicon = $2 ORDER BY seq LIMIT $3, $4;";
    let res: Result<Vec<(u32, String)>, sqlx::Error> = sqlx::query(query)
        .bind(seq)
        .bind(table)
        .bind(page * limit as i32)
        .bind(limit)
        .map(|rec: SqliteRow| (rec.get("seq"), rec.get("word")))
        .fetch_all(pool)
        .await;

    res
}
