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

use actix_web::{web, Error as AWError};
use failure::Error;
use futures::{Future, TryFutureExt};
use rusqlite::{Statement};
use serde::{Deserialize, Serialize};
use percent_encoding::percent_decode_str;

pub type Pool = r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>;
pub type Connection = r2d2::PooledConnection<r2d2_sqlite::SqliteConnectionManager>;

type PhilologusWordsResult = Result<Vec<QueryResults>, rusqlite::Error>;

#[derive(Debug, Serialize, Deserialize,Clone)]
pub enum PhilologusWords {
    GreekDefs { seq: i32, def: String },
}

//[{"i":1,"r":["Α α",1,0]},
// {"i":2,"r":["ἀ-",2,0]},
#[derive(Debug, Serialize, Deserialize,Clone)]
pub struct QueryResults { 
    pub i: i32, 
    pub r: (String,u32,u32)
}

pub fn execute(
    pool: &Pool,
    seq: u32,
    before_query:bool,
    lex: &str,
    page: i32,
    limit: u32,
) -> impl Future<Output = Result<Vec<QueryResults>, AWError>> {
    let pool = pool.clone();
    let table = match lex {
        "ls" => "ZLATIN",
        "slater" => "ZSLATER",
        _ => "ZGREEK"
    };

    web::block(move || {
        let before;
        if before_query {
            before = get_before(pool.get().unwrap(), table, seq, page, limit);
        }
        else {
            before = get_equal_and_after(pool.get().unwrap(), table, seq, page, limit);
        } 
        before.map_err(Error::from)
    })
    .map_err(AWError::from)
}

pub fn execute_get_seq(
    pool: &Pool,
    lex: &str,
    word: &str,
    prefix: &str,
) -> impl Future<Output = Result<u32, AWError>> {
    let pool = pool.clone();
    let table = match lex {
        "ls" => "ZLATIN",
        "slater" => "ZSLATER",
        _ => "ZGREEK"
    };

    let prefix_local = prefix.to_owned();
    let word_local = word.to_owned();

    web::block(move || {
        if word_local == "" {
            get_seq_by_prefix(pool.get().unwrap(), &table, &prefix_local).map_err(Error::from)
        }
        else {
            get_seq_by_word(pool.get().unwrap(), &table, &word_local).map_err(Error::from)
        }
    })
    .map_err(AWError::from)
}

pub fn execute_get_def(
    pool: &Pool,
    lex: &str,
    id: Option<u32>,
    word: &Option<String>,
) -> impl Future<Output = Result<(String,String,String,u32), AWError>> {
    let pool = pool.clone();
    let table = match lex {
        "ls" => "ZLATIN",
        "slater" => "ZSLATER",
        _ => "ZGREEK"
    };
    let wordid = id;
    let word2 = word.clone();

    web::block(move || {

        let d;

        if !word2.is_none() {
            d = get_def_by_word(pool.get().unwrap(), &table, word2.unwrap().as_str() );
        }
        else { //if !wordid.is_none() {
            d = get_def_by_seq(pool.get().unwrap(), &table, wordid.unwrap() );
        }

        /*match d {
            Ok(ref d) => println!("ok"),
            Err(ref err) => println!("error: {}", err),
        }*/
        d.map_err(Error::from)
    })
    .map_err(AWError::from)
}

fn get_def_by_word(conn: Connection, table:&str, word:&str) -> Result<(String,String,String,u32), rusqlite::Error> {
    let decoded_word = percent_decode_str(word).decode_utf8()?;
    let query = format!("{}{}{}{}{}", "SELECT word,sortword,def,seq FROM ", table, " WHERE word = '", decoded_word, "' LIMIT 1;");
    conn.query_row(&query, [], |r| Ok( (r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)? )) )
}
fn get_def_by_seq(conn: Connection, table:&str, id:u32) -> Result<(String,String,String,u32), rusqlite::Error> {
    let query = format!("{}{}{}{}{}", "SELECT word,sortword,def,seq FROM ", table, " WHERE seq = ", id, " LIMIT 1;");
    conn.query_row(&query, [], |r| Ok( (r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)? )) )
}

//, SEQ_COL, $table, UNACCENTED_COL, $word, STATUS_COL, UNACCENTED_COL);
fn get_seq_by_prefix(conn: Connection, table:&str, word:&str) -> Result<u32, rusqlite::Error> {
    let query = format!("{}{}{}{}{}", "SELECT seq FROM ", table, " WHERE sortword >= '", word, "' ORDER BY sortword LIMIT 1;");
    let res = conn.query_row(&query, [], |r| Ok(r.get(0)) );
    
    match res {
        Ok(res) => return res,
        Err(_) => {
            let query = format!("{}{}{}", "SELECT MAX(seq) FROM ", table, " LIMIT 1;");
            return conn.query_row(&query, [], |r| r.get(0) );
        }
    }
}

//, SEQ_COL, $table, UNACCENTED_COL, $word, STATUS_COL, UNACCENTED_COL);
fn get_seq_by_word(conn: Connection, table:&str, word:&str) -> Result<u32, rusqlite::Error> {
    let decoded_word = percent_decode_str(word).decode_utf8()?;
    let query = format!("{}{}{}{}{}", "SELECT seq FROM ", table, " WHERE word = '", decoded_word, "' LIMIT 1;");
    let res = conn.query_row(&query, [], |r| Ok(r.get(0)) );
    
    match res {
        Ok(res) => return res,
        Err(_) => {
            return Ok(1);
        }
    }
}

//, ID_COL, WORD_COL, $table, $tagJoin, SEQ_COL, $middleSeq, STATUS_COL, $tagwhere, SEQ_COL, $req->limit * $req->page * -1, $req->limit);
fn get_before(conn: Connection, table:&str, seq: u32, page: i32, limit: u32) -> PhilologusWordsResult {
    let query = format!("{}{}{}{}{}{}{}{}{}", "SELECT seq,word FROM ", table, " WHERE seq < ", seq, " ORDER BY seq DESC LIMIT ", page * limit as i32 * -1, ",", limit, ";");
    let stmt = conn.prepare(&query)?;
    get_word_res(stmt)
}

//, ID_COL, WORD_COL, $table, $tagJoin, SEQ_COL, $middleSeq, STATUS_COL, $tagwhere, SEQ_COL, $req->limit * $req->page, $req->limit);
fn get_equal_and_after(conn: Connection, table:&str, seq: u32, page: i32, limit: u32) -> PhilologusWordsResult {
    let query = format!("{}{}{}{}{}{}{}{}{}", "SELECT seq,word FROM ", table, " WHERE seq >= ", seq, " ORDER BY seq LIMIT ", page * limit as i32, ",", limit, ";");
    let stmt = conn.prepare(&query)?;
    get_word_res(stmt)
}

fn get_word_res(mut statement: Statement) -> PhilologusWordsResult {
    statement
        .query_map([], |row| {
            let a = (row.get(1)?, row.get(0)?, 0);
            Ok(QueryResults {
                i: row.get(0)?,
                r: a,
            })
        })
        .and_then(Iterator::collect)
}
