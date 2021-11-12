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
use r2d2_sqlite::{self, SqliteConnectionManager};

use actix_files::NamedFile;
use std::path::PathBuf;


mod db;
use db::{Pool,QueryInfo,WordQuery,DefInfo,GreekWords};
use serde::{Deserialize, Serialize};

/*
{"error":"","wtprefix":"test1","nocache":"1","container":"test1Container","requestTime":"1635643672625","selectId":"32","page":"0","lastPage":"0","lastPageUp":"1","scroll":"32","query":"","arrOptions":[{"i":1,"r":["Α α",1,0]},{"i":2,"r":["ἀ-",2,0]},{"i":3,"r":["ἀ-",3,0]},{"i":4,"r":["ἆ",4,0]}...
*/

#[derive(Debug, Serialize, Deserialize, Clone)]
struct JsonResponse {
    bstart:i32,
    bend:i32,
    astart:i32,
    aend:i32,
    #[serde(rename(serialize = "selectId"))]
    select_id: String,
    error: String,
    wtprefix: String,
    nocache: String,
    container: String,
    #[serde(rename(serialize = "requestTime"))]
    request_time: String,
    page: String,
    #[serde(rename(serialize = "lastPage"))]
    last_page: String,
    #[serde(rename(serialize = "lastPageUp"))]
    lastpage_up: String,
    scroll: String,
    query: String,
    #[serde(rename(serialize = "arrOptions"))]
    arr_options: Vec<GreekWords>
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct DefResponse {
    #[serde(rename(serialize = "principalPart"))]
    principal_part: String,
    def: String,
    #[serde(rename(serialize = "defName"))]
    def_name: String,
    word: String,
    #[serde(rename(serialize = "unaccentedWord"))]
    unaccented_word: String,
    lemma: String,
    #[serde(rename(serialize = "requestTime"))]
    request_time: String,
    status: String,
    lexicon: String,
    word_id: String,
    wordid: String,
    method: String,
}

//http://127.0.0.1:8088/philwords?n=101&idprefix=test1&x=0.1627681205837177&requestTime=1635643672625&page=0&mode=context&query={%22regex%22:%220%22,%22lexicon%22:%22lsj%22,%22tag_id%22:%220%22,%22root_id%22:%220%22,%22wordid%22:%22%CE%B1%CE%B1%CF%84%CE%BF%CF%832%22,%22w%22:%22%22}

#[allow(clippy::eval_order_dependence)]
async fn philologus_words((db, info): (web::Data<Pool>, web::Query<QueryInfo>)) -> Result<HttpResponse, AWError> {
    let p: WordQuery = serde_json::from_str(&info.query)?;
    
    let seq = db::execute_get_seq(&db,&p).await?;
    let mut before_rows = vec![];
    let mut after_rows = vec![];
    if info.page <= 0 {
        before_rows = db::execute(&db, seq, true, &p, info.page, info.n).await?;
        if info.page == 0 { //only reverse if page 0. if < 0, each row is inserted under top of container one-by-one in order
            before_rows.reverse();
        }
    }
    if info.page >= 0 {
        after_rows = db::execute(&db, seq, false, &p, info.page, info.n).await?;
    }

    let mut scroll = "".to_string();
    let mut vlast_page = "0".to_string();
    let mut vlast_page_up = "0".to_string();
    if info.page == 0 {
        if before_rows.len() < info.n as usize
        {
            vlast_page_up = "1".to_string();
        }
        else if after_rows.len() < info.n as usize
        {
            vlast_page = "1".to_string();
        }
    }

    if p.w == "" && info.page == 0 {
        scroll = "top".to_string();
    }

    if scroll == "" {
        scroll = seq.to_string();
    }

    let mut b_start = -1;
    let mut b_end = -1;
    let mut a_start = -1;
    let mut a_end = -1;

    if before_rows.len() > 0 {
        b_start = before_rows[0].i;
        b_end = before_rows[before_rows.len()-1].i;
    }

    if after_rows.len() > 0 {
        a_start = after_rows[0].i;
        a_end = after_rows[after_rows.len()-1].i;
    }

    let result = [before_rows, after_rows].concat();

    let re = Regex::new(r"[0-9]").unwrap();
    let result_stripped = result.into_iter().map( |mut row| { row.r.0 = re.replace_all(&row.r.0, "").to_string(); row }).collect();

    let res = JsonResponse {
        bstart: b_start,
        bend: b_end,
        astart: a_start,
        aend: a_end,
        select_id: seq.to_string(),
        error: "".to_owned(),
        wtprefix: info.idprefix.clone(),
        nocache: "0".to_owned(),
        container: format!("{}Container", info.idprefix),
        request_time: info.request_time.to_string(),
        page: info.page.to_string(),
        last_page: vlast_page,
        lastpage_up: vlast_page_up,
        scroll: scroll,
        query: p.w.to_owned(),
        arr_options: result_stripped
    };

    Ok(HttpResponse::Ok().json(res))
}

//http://127.0.0.1:8088/wordservjson.php?id=110628&lexicon=lsj&skipcache=0&addwordlinks=0&x=0.7049151126608002
//{"principalParts":"","def":"...","defName":"","word":"γεοῦχος","unaccentedWord":"γεουχοσ","lemma":"γεοῦχος","requestTime":0,"status":"0","lexicon":"lsj","word_id":"22045","wordid":"γεουχοσ","method":"setWord"}

#[allow(clippy::eval_order_dependence)]
async fn philologus_defs((db, info): (web::Data<Pool>, web::Query<DefInfo>)) -> Result<HttpResponse, AWError> {
    //if let Some(o) = &info.lex {
        //println!("lex: {}", path.into_inner());
    //}
    let def = db::execute_get_def(&db, &info.lexicon, info.id, &info.wordid).await?;

    let res = DefResponse {
        principal_part: "".to_string(),
        def: def.2.to_string(),
        def_name: "".to_string(),
        word: def.0.to_string(),
        unaccented_word: def.1.to_string(),
        lemma: "".to_string(),
        request_time: "0".to_string(),
        status: "0".to_string(),
        lexicon: info.lexicon.to_string(),
        word_id: def.3.to_string(),
        wordid: def.0.to_string(),
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

    let manager = SqliteConnectionManager::file( db_path );
    let pool = Pool::new(manager).unwrap();
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
            .data(pool.clone())
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

