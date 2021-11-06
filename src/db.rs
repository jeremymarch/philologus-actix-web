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
use rusqlite::{Statement, NO_PARAMS};
use serde::{Deserialize, Serialize};

pub type Pool = r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>;
pub type Connection = r2d2::PooledConnection<r2d2_sqlite::SqliteConnectionManager>;

type PhilologusWordsResult = Result<Vec<GreekWords>, rusqlite::Error>;

//[{"i":1,"r":["Α α",1,0]},
// {"i":2,"r":["ἀ-",2,0]},
#[derive(Debug, Serialize, Deserialize,Clone)]
pub enum PhilologusWords {
    GreekDefs { seq: i32, def: String },
}

#[derive(Debug, Serialize, Deserialize,Clone)]
pub struct GreekWords { 
    pub i: i32, 
    pub r: (String,u32,u32)
}

#[derive(Deserialize)]
pub struct WordQuery {
    pub regex:String,
    pub lexicon:String,
    pub tag_id:String,
    pub root_id:String,
    pub wordid:Option<String>,
    pub w:String,
}

#[derive(Deserialize)]
pub struct QueryInfo {
    pub n: u32,
    pub idprefix: String,
    pub x:String,
    #[serde(rename(deserialize = "requestTime"))]
    pub request_time:u64,
    pub page:u32,
    pub mode:String,
    pub query:String,//WordQuery,
}

//http://127.0.0.1:8080/wordservjson.php?id=110628&lexicon=lsj&skipcache=0&addwordlinks=0&x=0.7049151126608002

#[derive(Deserialize)]
pub struct DefInfo {
    pub id: u32,
    pub lexicon: String,
    pub skipcache:u32,
    pub addwordlinks:u32,
}

pub fn execute(
    pool: &Pool,
    seq: u32,
    before_query:bool,
    q: &WordQuery,
) -> impl Future<Output = Result<Vec<GreekWords>, AWError>> {
    let pool = pool.clone();
    let table = match q.lexicon.as_ref() {
        "ls" => "ZLATIN",
        "slater" => "ZSLATER",
        _ => "ZGREEK"
    };

    web::block(move || {
        let before;
        if before_query {
            before = get_before(pool.get().unwrap(), table, seq);
        }
        else {
            before = get_equal_and_after(pool.get().unwrap(), table, seq);
        } 
        before.map_err(Error::from)
    })
    .map_err(AWError::from)
}

pub fn execute_get_seq(
    pool: &Pool,
    q: &WordQuery,
) -> impl Future<Output = Result<u32, AWError>> {
    let pool = pool.clone();
    let table = match q.lexicon.as_ref() {
        "ls" => "ZLATIN",
        "slater" => "ZSLATER",
        _ => "ZGREEK"
    };

    let word = q.w.clone();
    web::block(move || {

        //let result = get_words(&pool, "ZGREEK", "γερ");
        let seq:u32 = get_seq(pool.get().unwrap(), &table, &word);
        //let before = get_before(pool.clone().get().unwrap(), table, seq);//.unwrap();
        //let after = get_equal_and_after(pool.get().unwrap(), table, seq).unwrap();
        //before.reverse();
        //let result = Ok([before.as_slice(), after.as_slice()].concat());//.map_err(Error::from) 
        Ok(seq).map_err(|_:u32| ())
    })
    .map_err(AWError::from)
}

pub fn execute_get_def(
    pool: &Pool,
    q: &DefInfo,
) -> impl Future<Output = Result<(String,String,String), AWError>> {
    let pool = pool.clone();
    let table = match q.lexicon.as_ref() {
        "ls" => "ZLATIN",
        "slater" => "ZSLATER",
        _ => "ZGREEK"
    };
    let id = q.id;
    //let word = q.w.clone();
    web::block(move || {

        let d = get_def(pool.get().unwrap(), &table, id);

        Ok(d).map_err(|_:u32| ())
    })
    .map_err(AWError::from)
}


fn get_def(conn: Connection, table:&str, id:u32) -> (String,String,String) {
    let query = format!("{}{}{}{}{}", "SELECT word,sortword,def FROM ", table, " WHERE seq = ", id, " LIMIT 1;");
    let def: (String,String,String) = conn.query_row(&query, NO_PARAMS, |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?)) ).unwrap();
    def
}

//, SEQ_COL, $table, UNACCENTED_COL, $word, STATUS_COL, UNACCENTED_COL);
fn get_seq(conn: Connection, table:&str, word:&str) -> u32 {
    let query = format!("{}{}{}{}{}", "SELECT seq FROM ", table, " WHERE sortword >= '", word, "' ORDER BY sortword LIMIT 1;");
    let seq: u32 = conn.query_row(&query, NO_PARAMS, |r| r.get(0)).unwrap();
    seq
}

//, ID_COL, WORD_COL, $table, $tagJoin, SEQ_COL, $middleSeq, STATUS_COL, $tagwhere, SEQ_COL, $req->limit * $req->page * -1, $req->limit);
fn get_before(conn: Connection, table:&str, seq: u32) -> PhilologusWordsResult {
    let query = format!("{}{}{}{}{}", "SELECT seq,word FROM ", table, " WHERE seq < ", seq, " ORDER BY seq DESC LIMIT 0,20;");
    let stmt = conn.prepare(&query)?;
    get_word_res(stmt)
}

//, ID_COL, WORD_COL, $table, $tagJoin, SEQ_COL, $middleSeq, STATUS_COL, $tagwhere, SEQ_COL, $req->limit * $req->page, $req->limit);
fn get_equal_and_after(conn: Connection, table:&str, seq: u32) -> PhilologusWordsResult {
    let query = format!("{}{}{}{}{}", "SELECT seq,word FROM ", table, " WHERE seq >= ", seq, " ORDER BY seq LIMIT 0,20;");
    let stmt = conn.prepare(&query)?;
    get_word_res(stmt)
}

fn get_word_res(mut statement: Statement) -> PhilologusWordsResult {
    statement
        .query_map(NO_PARAMS, |row| {
            let a = (row.get(1)?, row.get(0)?, 0);
            Ok(GreekWords {
                i: row.get(0)?,
                r: a,
            })
        })
        .and_then(Iterator::collect)
}
