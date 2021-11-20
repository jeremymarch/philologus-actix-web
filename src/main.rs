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

use std::io;
use regex::Regex;
use actix_files as fs;
use actix_web::{middleware, web, App, Error as AWError, HttpResponse, HttpRequest, HttpServer, Result};
use sqlx::SqlitePool;

use actix_files::NamedFile;
use std::path::PathBuf;

mod db;
use db::{QueryResults};
use crate::db::get_seq_by_word;
use crate::db::get_def_by_word;
use crate::db::get_def_by_seq;
use crate::db::get_before;
use crate::db::get_equal_and_after;
use serde::{Deserialize, Serialize};
use crate::db::get_seq_by_prefix;

/*
{"error":"","wtprefix":"test1","nocache":"1","container":"test1Container","requestTime":"1635643672625","selectId":"32","page":"0","lastPage":"0","lastPageUp":"1","scroll":"32","query":"","arrOptions":[{"i":1,"r":["Α α",1,0]},{"i":2,"r":["ἀ-",2,0]},{"i":3,"r":["ἀ-",3,0]},{"i":4,"r":["ἆ",4,0]}...
*/

#[derive(Debug, Serialize, Deserialize, Clone)]
struct QueryResponse {
    #[serde(rename(serialize = "selectId"))]
    select_id: u32,
    error: String,
    wtprefix: String,
    nocache: u8,
    container: String,
    #[serde(rename(serialize = "requestTime"))]
    request_time: u64,
    page: i32,
    #[serde(rename(serialize = "lastPage"))]
    last_page: u8,
    #[serde(rename(serialize = "lastPageUp"))]
    lastpage_up: u8,
    scroll: String,
    query: String,
    #[serde(rename(serialize = "arrOptions"))]
    arr_options: Vec<QueryResults>
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct DefResponse {
    #[serde(rename(serialize = "principalParts"))]
    principal_parts: String,
    def: String,
    #[serde(rename(serialize = "defName"))]
    def_name: String,
    word: String,
    #[serde(rename(serialize = "unaccentedWord"))]
    unaccented_word: String,
    lemma: String,
    #[serde(rename(serialize = "requestTime"))]
    request_time: u64,
    status: String,
    lexicon: String,
    word_id: u32,
    wordid: String,
    method: String,
}

#[derive(Deserialize)]
pub struct QueryRequest {
    pub n: u32,
    pub idprefix: String,
    pub x: String,
    #[serde(rename(deserialize = "requestTime"))]
    pub request_time: u64,
    pub page: i32,
    pub mode: String,
    pub query: String,//WordQuery,
    pub lex: Option<String>,
}

#[derive(Deserialize)]
pub struct WordQuery {
    pub regex: String,
    pub lexicon: String,
    pub tag_id: String,
    pub root_id: String,
    pub wordid: Option<String>,
    pub w: String,
}

//http://127.0.0.1:8080/wordservjson.php?id=110628&lexicon=lsj&skipcache=0&addwordlinks=0&x=0.7049151126608002

#[derive(Deserialize)]
pub struct DefRequest {
    pub id: Option<u32>,
    pub wordid: Option<String>,
    pub lexicon: String,
    pub skipcache: u32,
    pub addwordlinks: u32,
    pub lex: Option<String>,
}

//http://127.0.0.1:8088/philwords?n=101&idprefix=test1&x=0.1627681205837177&requestTime=1635643672625&page=0&mode=context&query={%22regex%22:%220%22,%22lexicon%22:%22lsj%22,%22tag_id%22:%220%22,%22root_id%22:%220%22,%22wordid%22:%22%CE%B1%CE%B1%CF%84%CE%BF%CF%832%22,%22w%22:%22%22}

#[allow(clippy::eval_order_dependence)]
async fn philologus_words((info, req): (web::Query<QueryRequest>, HttpRequest)) -> Result<HttpResponse, AWError> {
    let db = req.app_data::<SqlitePool>().unwrap();

    let p: WordQuery = serde_json::from_str(&info.query)?;

    let wordid = p.wordid.unwrap_or_else(|| "".to_string());
    
    let table = match p.lexicon.as_str() {
        "ls" => "ZLATIN",
        "slater" => "ZSLATER",
        _ => "ZGREEK"
    };
    let seq;
    if wordid == "" {
        seq = get_seq_by_prefix(&db, &table, &p.w).await?;
    }
    else {
        seq = get_seq_by_word(&db, &table, &wordid).await?;
    }


    let mut before_rows = vec![];
    let mut after_rows = vec![];
    if info.page <= 0 {
        
        before_rows = get_before(&db, table, seq, info.page, info.n).await?;
        if info.page == 0 { //only reverse if page 0. if < 0, each row is inserted under top of container one-by-one in order
            before_rows.reverse();
        }
    }
    if info.page >= 0 {
        after_rows = get_equal_and_after(&db, table, seq, info.page, info.n).await?;
    }

    let mut vlast_page = 0;
    let mut vlast_page_up = 0;
    if info.page == 0 {
        if before_rows.len() < info.n as usize {
            vlast_page_up = 1;
        }
        else if after_rows.len() < info.n as usize {
            vlast_page = 1;
        }
    }

    let result = [before_rows, after_rows].concat();

    //strip any numbers from end of string
    let re = Regex::new(r"[0-9]").unwrap();
    let result_stripped = result.into_iter().map( |mut row| { row.r.0 = re.replace_all(&row.r.0, "").to_string(); row }).collect();

    let res = QueryResponse {
        select_id: seq,
        error: "".to_owned(),
        wtprefix: info.idprefix.clone(),
        nocache: if wordid == "" { 0 } else { 1 }, //prevents caching when queried by wordid in url
        container: format!("{}Container", info.idprefix),
        request_time: info.request_time,
        page: info.page,
        last_page: vlast_page,
        lastpage_up: vlast_page_up,
        scroll: if p.w == "" && info.page == 0 && seq == 1 { "top".to_string() } else { "".to_string() }, //scroll really only needs to return top
        query: p.w.to_owned(),
        arr_options: result_stripped
    };

    Ok(HttpResponse::Ok().json(res))
}

//http://127.0.0.1:8088/wordservjson.php?id=110628&lexicon=lsj&skipcache=0&addwordlinks=0&x=0.7049151126608002
//{"principalParts":"","def":"...","defName":"","word":"γεοῦχος","unaccentedWord":"γεουχοσ","lemma":"γεοῦχος","requestTime":0,"status":"0","lexicon":"lsj","word_id":"22045","wordid":"γεουχοσ","method":"setWord"}

#[allow(clippy::eval_order_dependence)]
async fn philologus_defs((info, req): (web::Query<DefRequest>, HttpRequest)) -> Result<HttpResponse, AWError> {
    let db = req.app_data::<SqlitePool>().unwrap();
    
    let table = match info.lexicon.as_str() {
        "ls" => "ZLATIN",
        "slater" => "ZSLATER",
        _ => "ZGREEK"
    };

    let def;

    if !info.wordid.is_none() {
        def = get_def_by_word(&db, &table, &info.wordid.as_ref().unwrap() ).await?;
    }
    else { //if !wordid.is_none() {
        def = get_def_by_seq(&db, &table, info.id.unwrap() ).await?;
    }

    let res = DefResponse {
        principal_parts: "".to_string(),
        def: def.as_ref().unwrap().def.to_string(),
        def_name: "".to_string(),
        word: def.as_ref().unwrap().word.to_string(),
        unaccented_word: def.as_ref().unwrap().sortword.to_string(),
        lemma: "".to_string(),
        request_time: 0,
        status: "0".to_string(),
        lexicon: info.lexicon.to_string(),
        word_id: def.as_ref().unwrap().seq,
        wordid: def.as_ref().unwrap().word.to_string(),
        method: "setWord".to_string()
    };

    Ok(HttpResponse::Ok().json(res))
}

async fn index(_req: HttpRequest) -> Result<NamedFile> {
    let path: PathBuf = "static/index.html".parse().unwrap();
    Ok(NamedFile::open(path)?)
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let db_path = std::env::var("PHILOLOGUS_DB_PATH")
                   .unwrap_or_else(|_| panic!("Environment variable for sqlite path not set: PHILOLOGUS_DB_PATH."));

    let db_pool = SqlitePool::connect(&db_path).await.expect("Could not connect to db.");
/*
    let error_handlers = ErrorHandlers::new()
            .handler(
                http::StatusCode::INTERNAL_SERVER_ERROR,
                api::internal_server_error,
            )
            .handler(http::StatusCode::BAD_REQUEST, api::bad_request)
            .handler(http::StatusCode::NOT_FOUND, api::not_found);
*/
    HttpServer::new(move || {
        App::new()
            .app_data(db_pool.clone())
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            //.wrap(error_handlers)
            .service(
                web::resource("/{lex}/wtgreekserv.php")
                    .route(web::get().to(philologus_words)),
            )
            .service(
                web::resource("/{lex}/wordservjson.php")
                    .route(web::get().to(philologus_defs)),
            )
            .service(
                web::resource("/{lex}/{word}")
                    .route(web::get().to(index))) //requesting page from a word link, order of services matters
            .service(
                web::resource("/wordservjson.php")
                    .route(web::get().to(philologus_defs)),
            )
            .service(
                web::resource("/wtgreekserv.php")
                    .route(web::get().to(philologus_words)),
            )
            .service(fs::Files::new("/", "./static").prefer_utf8(true).index_file("index.html"))
    })
    .bind("0.0.0.0:8088")?
    .run()
    .await
}


#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App};

    #[actix_rt::test]
    async fn test_index_get() {
        let mut app = test::init_service(App::new().route("/", web::get().to(index))).await;
        let req = test::TestRequest::with_header("content-type", "text/plain").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_rt::test]
    async fn test_index_post() {
        let mut app = test::init_service(App::new().route("/", web::get().to(index))).await;
        let req = test::TestRequest::post().uri("/").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_client_error());
    }
}

