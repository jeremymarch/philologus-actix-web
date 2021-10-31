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

type PhilologusWordsResult = Result<Vec<PhilologusWords>, rusqlite::Error>;

#[derive(Debug, Serialize, Deserialize,Clone)]
pub enum PhilologusWords {
    GreekWords { i: i32, word: String },
    GreekDefs { seq: i32, def: String },
}

#[derive(Deserialize)]
pub struct WordQuery {
    pub regex:String,
    pub lexicon:String,
    pub tag_id:String,
    pub root_id:String,
    pub wordid:String,
    pub w:String,
}

#[derive(Deserialize)]
pub struct Info {
    pub n: u32,
    pub idprefix: String,
    pub x:String,
    pub requestTime:u64,
    pub page:u32,
    pub mode:String,
    pub query:String,//WordQuery,
}

pub fn execute(
    pool: &Pool,
    seq: u32,
    before_query:bool
) -> impl Future<Output = Result<Vec<PhilologusWords>, AWError>> {
    let pool = pool.clone();
    web::block(move || {

        //let result = get_words(&pool, "ZGREEK", "γερ");
        let table = "ZGREEK";
        let before;
        if before_query {
            before = get_before(pool.get().unwrap(), table, seq);//.unwrap();
        }
        else {
            before = get_equal_and_after(pool.get().unwrap(), table, seq);//.unwrap();  
        }
        //
        //before.reverse();
        //let result = Ok([before.as_slice(), after.as_slice()].concat());//.map_err(Error::from)   
        before.map_err(Error::from)
    })
    .map_err(AWError::from)
}

pub fn execute_get_seq(
    pool: &Pool,
) -> impl Future<Output = Result<u32, AWError>> {
    let pool = pool.clone();
    web::block(move || {

        //let result = get_words(&pool, "ZGREEK", "γερ");
        let table = "ZGREEK";
        let word = "γερ";
        let seq:u32 = get_seq(pool.get().unwrap(), table, word);
        //let before = get_before(pool.clone().get().unwrap(), table, seq);//.unwrap();
        //let after = get_equal_and_after(pool.get().unwrap(), table, seq).unwrap();
        //before.reverse();
        //let result = Ok([before.as_slice(), after.as_slice()].concat());//.map_err(Error::from) 
        Ok(seq).map_err(|_:u32| ())
    })
    .map_err(AWError::from)
}
/*
fn get_words(conn: &Pool, table: &str, word:&str) -> PhilologusWordsResult {
    let seq = get_seq(conn.get().unwrap(), table, word);
    let mut before = get_before(conn.get().unwrap(), table, seq).unwrap();
    let after = get_equal_and_after(conn.get().unwrap(), table, seq).unwrap();
    before.reverse();
    Ok([before.as_slice(), after.as_slice()].concat())
}
*/
//, SEQ_COL, $table, UNACCENTED_COL, $word, STATUS_COL, UNACCENTED_COL);
fn get_seq(conn: Connection, table:&str, word:&str) -> u32 {
    let query = format!("{}{}{}{}{}", "SELECT seq FROM ", table, " WHERE sortword >= '", word, "' ORDER BY sortword LIMIT 1;");
    //let stmt = conn.prepare(&query);
    //get_seq_res(stmt, word)

    let seq: u32 = conn.query_row(&query, NO_PARAMS, |r| r.get(0)).unwrap();
    println!("seq: {}", seq);
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
            Ok(PhilologusWords::GreekWords {
                i: row.get(0)?,
                word: row.get(1)?,
            })
        })
        .and_then(Iterator::collect)
}

/*
{"error":"","wtprefix":"test1","nocache":"1","container":"test1Container","requestTime":"1635643672625","selectId":"32","page":"0","lastPage":"0","lastPageUp":"1","scroll":"32","query":"","arrOptions":[{"i":1,"r":["Α α",1,0]},{"i":2,"r":["ἀ-",2,0]},{"i":3,"r":["ἀ-",3,0]},{"i":4,"r":["ἆ",4,0]}...
*/


