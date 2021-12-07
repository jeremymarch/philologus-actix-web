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

pub async fn insert_hc_log(pool: &SqlitePool,
    answer_text: &str,
    expected_answer: &str,
    is_correct: &str,
    answer_seconds: &str,
    answer_timed_out: &str,
    multiple_forms_pressed: &str,
    lives: &str,
    _score: &str,
    verb_id: &str,
    person: &str,
    number: &str,
    tense: &str,
    voice: &str,
    mood: &str,
    appversion: &str,
    device: &str,
    agent: &str,
    screen: &str,
    _accessdate: &str,
    error: &str,
    time_stamp_ms: &str,
    ip: &str) -> Result<u32, sqlx::Error> {

    let move_id = "NULL";
    let game_id = "1";
    let is_game = if lives != "-1" { 1 } else { 0 };
    let answer_seconds2 = "0";
    let ask_player_id = "1";
    let answer_player_id = "1";


    let query = format!("INSERT INTO hcmoves (moveID,gameID,verbID,person,number,tense,voice,mood,answerIsCorrect,answerText,expectedAnswer,mfPressed,isGame,answerSeconds,answerSeconds2,answerTimedOut,askPlayerID,askTimestamp,askIP,askDevice,askScreen,askOSVersion,askAppVersion,askError,answerPlayerID,answerTimestamp,answerIP,answerDevice,answerScreen,answerOSVersion,answerAppVersion,answerError,lastUpdated ) VALUES ({move_id},{game_id},{verb_id},{person},{number},{tense},{voice},{mood},{is_correct},'{answer_text}','{expected_answer}',{multiple_forms_pressed},{is_game},'{answer_seconds}',{answer_seconds2},{answer_timed_out},{ask_player_id},{ask_timestamp},'{ask_ip}','{ask_device}','{ask_screen}','{ask_os_version}','{ask_app_version}','{ask_error}',{answer_player_id},{answer_timestamp},'{answer_ip}','{answer_device}','{answer_screen}','{answer_os_version}','{answer_app_version}','{answer_error}', {last_updated});", 
    move_id = move_id,
    game_id = game_id,
    verb_id = verb_id,
    person = person,
    number = number,
    tense = tense,
    voice = voice,
    mood = mood,
    is_correct = is_correct,
    answer_text = answer_text,
    expected_answer = expected_answer,
    multiple_forms_pressed = multiple_forms_pressed,
    is_game = is_game,
    answer_seconds = answer_seconds,
    answer_seconds2 = answer_seconds2,
    answer_timed_out = answer_timed_out,
    ask_player_id = ask_player_id,
    ask_timestamp = time_stamp_ms,
    ask_ip = ip,
    ask_device = device,
    ask_screen = screen,
    ask_os_version = agent,
    ask_app_version = appversion,
    ask_error = error,
    answer_player_id = answer_player_id,
    answer_timestamp = time_stamp_ms,
    answer_ip = ip,
    answer_device = device,
    answer_screen = screen,
    answer_os_version = agent,
    answer_app_version = appversion,
    answer_error = error,
    last_updated = time_stamp_ms);

    //println!("query: {}", query);

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
