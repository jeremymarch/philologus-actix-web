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
use thiserror::Error;
use actix_web::{ ResponseError, http::StatusCode};
use percent_encoding::percent_decode_str;

use std::io;
use regex::Regex;
use actix_files as fs;
use actix_web::{middleware, web, App, Error as AWError, HttpResponse, HttpRequest, HttpServer, Result};
use sqlx::SqlitePool;

use actix_files::NamedFile;
use std::path::PathBuf;

mod db;
use crate::db::*;
use serde::{Deserialize, Serialize};

/*
{"error":"","wtprefix":"test1","nocache":"1","container":"test1Container","requestTime":"1635643672625","selectId":"32","page":"0","lastPage":"0","lastPageUp":"1","scroll":"32","query":"","arrOptions":[{"i":1,"r":["Α α",1,0]},{"i":2,"r":["ἀ-",2,0]},{"i":3,"r":["ἀ-",3,0]},{"i":4,"r":["ἆ",4,0]}...
*/

#[derive(Debug, Serialize, Deserialize, Clone)]
struct QueryResponse {
    #[serde(rename(serialize = "selectId"), rename(deserialize = "selectId"))]
    select_id: u32,
    error: String,
    wtprefix: String,
    nocache: u8,
    container: String,
    #[serde(rename(serialize = "requestTime"), rename(deserialize = "requestTime"))]
    request_time: u64,
    page: i32, //can be negative for pages before
    #[serde(rename(serialize = "lastPage"), rename(deserialize = "lastPage"))]
    last_page: u8,
    #[serde(rename(serialize = "lastPageUp"), rename(deserialize = "lastPageUp"))]
    lastpage_up: u8,
    scroll: String,
    query: String,
    #[serde(rename(serialize = "arrOptions"), rename(deserialize = "arrOptions"))]
    arr_options: Vec<(String,u32)>
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct DefResponse {
    #[serde(rename(serialize = "principalParts"), rename(deserialize = "principalParts"))]
    principal_parts: Option<String>,
    def: String,
    #[serde(rename(serialize = "defName"), rename(deserialize = "defName"))]
    def_name: Option<String>,
    word: String,
    #[serde(rename(serialize = "unaccentedWord"), rename(deserialize = "unaccentedWord"))]
    unaccented_word: String,
    lemma: Option<String>,
    #[serde(rename(serialize = "requestTime"), rename(deserialize = "requestTime"))]
    request_time: u64,
    status: String,
    lexicon: String,
    word_id: u32,
    method: String,
}

#[derive(Deserialize)]
pub struct QueryRequest {
    pub n: u32,
    pub idprefix: String,
    pub x: String,
    #[serde(rename(deserialize = "requestTime"))]
    pub request_time: u64,
    pub page: i32, //can be negative for pages before
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

    let query_params: WordQuery = serde_json::from_str(&info.query)?;
    
    let table = match query_params.lexicon.as_str() {
        "ls" => "ZLATIN",
        "slater" => "ZSLATER",
        _ => "ZGREEK"
    };
    
    let seq = if query_params.wordid.is_none() {
        get_seq_by_prefix(&db, &table, &query_params.w).await.map_err(map_sqlx_error)?
    }
    else {
        let decoded_word = percent_decode_str(query_params.wordid.as_ref().unwrap()).decode_utf8().map_err(map_utf8_error)?;
        get_seq_by_word(&db, &table, &decoded_word).await.map_err(map_sqlx_error)?
    };

    let mut before_rows = vec![];
    let mut after_rows = vec![];
    if info.page <= 0 {
        
        before_rows = get_before(&db, table, seq, info.page, info.n).await.map_err(map_sqlx_error)?;
        if info.page == 0 { //only reverse if page 0. if < 0, each row is inserted under top of container one-by-one in order
            before_rows.reverse();
        }
    }
    if info.page >= 0 {
        after_rows = get_equal_and_after(&db, table, seq, info.page, info.n).await.map_err(map_sqlx_error)?;
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

    let result_rows = [before_rows, after_rows].concat();

    //strip any numbers from end of string
    let re = Regex::new(r"[0-9]").unwrap();
    let result_rows_stripped = result_rows.into_iter().map( |mut row| { row.0 = re.replace_all(&row.0, "").to_string(); row }).collect();

    let res = QueryResponse {
        select_id: seq,
        error: "".to_owned(),
        wtprefix: info.idprefix.clone(),
        nocache: if query_params.wordid.is_none() { 0 } else { 1 }, //prevents caching when queried by wordid in url
        container: format!("{}Container", info.idprefix),
        request_time: info.request_time,
        page: info.page,
        last_page: vlast_page,
        lastpage_up: vlast_page_up,
        scroll: if query_params.w == "" && info.page == 0 && seq == 1 { "top".to_string() } else { "".to_string() }, //scroll really only needs to return top
        query: query_params.w.to_owned(),
        arr_options: result_rows_stripped
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

    let def_row = if !info.wordid.is_none() {
        let decoded_word = percent_decode_str( &info.wordid.as_ref().unwrap() ).decode_utf8().map_err(map_utf8_error)?;
        get_def_by_word(&db, &table, &decoded_word ).await.map_err(map_sqlx_error)?
    }
    else if !info.id.is_none() {
        get_def_by_seq(&db, &table, info.id.unwrap() ).await.map_err(map_sqlx_error)?
    }
    else {
        return Err(PhilologusError { code: StatusCode::INTERNAL_SERVER_ERROR, name: "philologus_defs error: word and id are both empty".to_string(), error: "philologus_defs error: word and id are both empty".to_string() })?
    };
    
    let res = DefResponse {
        principal_parts: None,
        def: def_row.def,
        def_name: None,
        word: def_row.word,
        unaccented_word: def_row.sortword,
        lemma: None,
        request_time: 0,
        status: "0".to_string(),
        lexicon: info.lexicon.to_string(),
        word_id: def_row.seq,
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
    https://github.com/SergioBenitez/Rocket/discussions/1989
    .journal_mode(SqliteJournalMode::Off)
    .read_only(true)
    */
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
                web::resource("/{lex}/query")
                    .route(web::get().to(philologus_words)),
            )
            .service(
                web::resource("/{lex}/item")
                    .route(web::get().to(philologus_defs)),
            )
            .service(
                web::resource("/{lex}/{word}")
                    .route(web::get().to(index))) //requesting page from a word link, order of services matters
            .service(
                web::resource("/item")
                    .route(web::get().to(philologus_defs)),
            )
            .service(
                web::resource("/query")
                    .route(web::get().to(philologus_words)),
            )
            .service(fs::Files::new("/", "./static").prefer_utf8(true).index_file("index.html"))
    })
    .bind("0.0.0.0:8088")?
    .run()
    .await
}

#[derive(Error, Debug)]
pub struct PhilologusError {
       code: StatusCode,
       name: String,
       error: String,
}

impl std::fmt::Display for PhilologusError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(fmt, "PhilologusError: {} {}: {}.", self.code.as_u16(), self.name, self.error)
    }
}

/*
impl From<sqlx::Error> for PhilologusError {
    fn from(err: sqlx::Error) -> PhilologusError {
        PhilologusError {
            message: Some(err.to_string()),
            err_type: CustomErrorType::DieselError,
        }
    }
}
*/

impl ResponseError for PhilologusError {
    fn error_response(&self) -> HttpResponse {
        let error_response = ErrorResponse {
            code: self.code.as_u16(),
            message: self.error.to_string(),
            error: self.error.to_string(),
        };
        HttpResponse::build(self.code).json(error_response)
    }
}

fn map_sqlx_error(e: sqlx::Error) -> PhilologusError {   
    match e {
        sqlx::Error::Configuration(e) => PhilologusError { code: StatusCode::INTERNAL_SERVER_ERROR, name: "sqlx error".to_string(), error: format!("sqlx Configuration: {}", e) },
        sqlx::Error::Database(e) => PhilologusError { code: StatusCode::INTERNAL_SERVER_ERROR, name: "sqlx error".to_string(), error: format!("sqlx Database: {}", e) },
        sqlx::Error::Io(e) => PhilologusError { code: StatusCode::INTERNAL_SERVER_ERROR, name: "sqlx error".to_string(), error: format!("sqlx Io: {}", e) },
        sqlx::Error::Tls(e) => PhilologusError { code: StatusCode::INTERNAL_SERVER_ERROR, name: "sqlx error".to_string(), error: format!("sqlx Tls: {}", e) },
        sqlx::Error::Protocol(e) => PhilologusError { code: StatusCode::INTERNAL_SERVER_ERROR, name: "sqlx error".to_string(), error: format!("sqlx Protocol: {}", e) },
        sqlx::Error::RowNotFound => PhilologusError { code: StatusCode::INTERNAL_SERVER_ERROR, name: "sqlx error".to_string(), error: "sqlx RowNotFound".to_string() },
        sqlx::Error::TypeNotFound { .. } => PhilologusError { code: StatusCode::INTERNAL_SERVER_ERROR, name: "sqlx error".to_string(), error: "sqlx TypeNotFound".to_string() },
        sqlx::Error::ColumnIndexOutOfBounds { .. } => PhilologusError { code: StatusCode::INTERNAL_SERVER_ERROR, name: "sqlx error".to_string(), error: "sqlx ColumnIndexOutOfBounds".to_string() },
        sqlx::Error::ColumnNotFound(e) => PhilologusError { code: StatusCode::INTERNAL_SERVER_ERROR, name: "sqlx error".to_string(), error: format!("sqlx ColumnNotFound: {}", e) },
        sqlx::Error::ColumnDecode { .. } => PhilologusError { code: StatusCode::INTERNAL_SERVER_ERROR, name: "sqlx error".to_string(), error: "sqlx ColumnDecode".to_string() },
        sqlx::Error::Decode(e) => PhilologusError { code: StatusCode::INTERNAL_SERVER_ERROR, name: "sqlx error".to_string(), error: format!("sqlx Decode: {}", e) },
        sqlx::Error::PoolTimedOut => PhilologusError { code: StatusCode::INTERNAL_SERVER_ERROR, name: "sqlx error".to_string(), error: "sqlx PoolTimeOut".to_string() },
        sqlx::Error::PoolClosed => PhilologusError { code: StatusCode::INTERNAL_SERVER_ERROR, name: "sqlx error".to_string(), error: "sqlx PoolClosed".to_string() },
        sqlx::Error::WorkerCrashed => PhilologusError { code: StatusCode::INTERNAL_SERVER_ERROR, name: "sqlx error".to_string(), error: "sqlx WorkerCrashed".to_string() },
        sqlx::Error::Migrate(e) => PhilologusError { code: StatusCode::INTERNAL_SERVER_ERROR, name: "sqlx error".to_string(), error: format!("sqlx Migrate: {}", e) },
        _ => PhilologusError { code: StatusCode::INTERNAL_SERVER_ERROR, name: "sqlx error".to_string(), error: "sqlx Unknown error".to_string() },
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    code: u16,
    error: String,
    message: String,
}

fn map_utf8_error(_e: std::str::Utf8Error) -> PhilologusError {   
    PhilologusError { code: StatusCode::INTERNAL_SERVER_ERROR, name: "percent_decode_str utf-8 error".to_string(), error: "percent_decode_str utf-8 error".to_string() }
}


#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App};
    use actix_web::body::{AnyBody};

    //use serde::{Serialize, Deserialize};
    //use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
    //use actix_web::http::header::ContentType;
/*
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
*/
    //convert response body to a utf8 string before deserializing json because this doesn't work as bytes
    trait BodyTest {
        fn as_str(&self) -> &str;
    }
    impl BodyTest for AnyBody {
        fn as_str(&self) -> &str {
            match self {
                AnyBody::Bytes(ref b) => std::str::from_utf8(b).unwrap(),
                _ => panic!()
            }
        }
    }

    #[actix_web::test]
    async fn test_query_paging() {
        let db_path = std::env::var("PHILOLOGUS_DB_PATH")
                   .unwrap_or_else(|_| panic!("Environment variable for sqlite path not set: PHILOLOGUS_DB_PATH."));

        let db_pool = SqlitePool::connect(&db_path).await.expect("Could not connect to db.");
        let mut app = test::init_service(
            App::new().app_data(db_pool.clone())
            .service(
                web::resource("/{lex}/query")
                    .route(web::get().to(philologus_words))
        )
        .service(
            web::resource("/{lex}/item")
                .route(web::get().to(philologus_defs)),
        )).await;

        let resp = test::TestRequest::get()
            .uri(r#"/lsj/query?n=101&idprefix=test1&x=0.795795025371805&requestTime=1637859894040&page=0&mode=context&query={%22regex%22:%220%22,%22lexicon%22:%22lsj%22,%22tag_id%22:%220%22,%22root_id%22:%220%22,%22w%22:%22%CF%86%CE%B1%22}"#)
            .send_request(&mut app).await;

        assert!(&resp.status().is_success());
        //println!("resp: {:?}", resp);
        let result: QueryResponse = serde_json::from_str( resp.response().body().as_str() ).unwrap();//test::read_body_json(req).await;
        //println!("res: {:?}", result);
        assert_eq!(result.arr_options.len(), 202);

        //empty query returns seq 1 for first row
        let resp = test::TestRequest::get()
            .uri(r#"/lsj/query?n=101&idprefix=test1&x=0.795795025371805&requestTime=1637859894040&page=0&mode=context&query={%22regex%22:%220%22,%22lexicon%22:%22lsj%22,%22tag_id%22:%220%22,%22root_id%22:%220%22,%22w%22:%22%22}"#)
            .send_request(&mut app).await;
        let result: QueryResponse = serde_json::from_str( resp.response().body().as_str() ).unwrap();
        assert_eq!(result.arr_options[0].1, 1);
        assert_eq!(result.arr_options[result.arr_options.len() - 1].1, 101);
        assert_eq!(result.lastpage_up, 1);
        assert_eq!(result.last_page, 0);
        assert_eq!(result.page, 0);

        //page 1
        let resp = test::TestRequest::get()
            .uri(r#"/lsj/query?n=101&idprefix=test1&x=0.795795025371805&requestTime=1637859894040&page=1&mode=context&query={%22regex%22:%220%22,%22lexicon%22:%22lsj%22,%22tag_id%22:%220%22,%22root_id%22:%220%22,%22w%22:%22%22}"#)
            .send_request(&mut app).await;
        let result: QueryResponse = serde_json::from_str( resp.response().body().as_str() ).unwrap();
        assert_eq!(result.arr_options[0].1, 102);
        assert_eq!(result.arr_options[result.arr_options.len() - 1].1, 202);
        assert_eq!(result.lastpage_up, 0);
        assert_eq!(result.last_page, 0);
        assert_eq!(result.page, 1);
        //page 2
        let resp = test::TestRequest::get()
            .uri(r#"/lsj/query?n=101&idprefix=test1&x=0.795795025371805&requestTime=1637859894040&page=2&mode=context&query={%22regex%22:%220%22,%22lexicon%22:%22lsj%22,%22tag_id%22:%220%22,%22root_id%22:%220%22,%22w%22:%22%22}"#)
            .send_request(&mut app).await;
        let result: QueryResponse = serde_json::from_str( resp.response().body().as_str() ).unwrap();
        assert_eq!(result.arr_options[0].1, 203);
        assert_eq!(result.arr_options[result.arr_options.len() - 1].1, 303);
        assert_eq!(result.lastpage_up, 0);
        assert_eq!(result.last_page, 0);
        assert_eq!(result.page, 2);

        //alpha query returns seq 1 for first row (exactly like empty query)
        let resp = test::TestRequest::get()
            .uri(r#"/lsj/query?n=101&idprefix=test1&x=0.795795025371805&requestTime=1637859894040&page=0&mode=context&query={%22regex%22:%220%22,%22lexicon%22:%22lsj%22,%22tag_id%22:%220%22,%22root_id%22:%220%22,%22w%22:%22%CE%B1%22}"#)
            .send_request(&mut app).await;
        let result: QueryResponse = serde_json::from_str( resp.response().body().as_str() ).unwrap();
        assert_eq!(result.arr_options[0].1, 1);
        assert_eq!(result.arr_options[result.arr_options.len() - 1].1, 101);
        assert_eq!(result.lastpage_up, 1);
        assert_eq!(result.last_page, 0);
        assert_eq!(result.page, 0);


        //last page
        let resp = test::TestRequest::get()
            .uri(r#"/lsj/query?n=101&idprefix=test1&x=0.795795025371805&requestTime=1637859894040&page=0&mode=context&query={%22regex%22:%220%22,%22lexicon%22:%22lsj%22,%22tag_id%22:%220%22,%22root_id%22:%220%22,%22w%22:%22%CF%89%CF%89%22}"#)
            .send_request(&mut app).await;
        let result: QueryResponse = serde_json::from_str( resp.response().body().as_str() ).unwrap();
        assert_eq!(result.arr_options[0].1, 116494);
        assert_eq!(result.arr_options[result.arr_options.len() - 1].1, 116596);
        assert_eq!(result.lastpage_up, 0);
        assert_eq!(result.last_page, 1);
        assert_eq!(result.page, 0);

        //last page - 1 (remember these pages are delivered in reverse order when page < 0)
        let resp = test::TestRequest::get()
            .uri(r#"/lsj/query?n=101&idprefix=test1&x=0.795795025371805&requestTime=1637859894040&page=-1&mode=context&query={%22regex%22:%220%22,%22lexicon%22:%22lsj%22,%22tag_id%22:%220%22,%22root_id%22:%220%22,%22w%22:%22%CF%89%CF%89%22}"#)
            .send_request(&mut app).await;
        let result: QueryResponse = serde_json::from_str( resp.response().body().as_str() ).unwrap();
        assert_eq!(result.arr_options[0].1, 116493);
        assert_eq!(result.arr_options[result.arr_options.len() - 1].1, 116393);
        assert_eq!(result.lastpage_up, 0);
        assert_eq!(result.last_page, 0);
        assert_eq!(result.page, -1);

        //last page - 2 (remember these pages are delivered in reverse order when page < 0)
        let resp = test::TestRequest::get()
            .uri(r#"/lsj/query?n=101&idprefix=test1&x=0.795795025371805&requestTime=1637859894040&page=-2&mode=context&query={%22regex%22:%220%22,%22lexicon%22:%22lsj%22,%22tag_id%22:%220%22,%22root_id%22:%220%22,%22w%22:%22%CF%89%CF%89%22}"#)
            .send_request(&mut app).await;
        let result: QueryResponse = serde_json::from_str( resp.response().body().as_str() ).unwrap();
        assert_eq!(result.arr_options[0].1, 116392);
        assert_eq!(result.arr_options[result.arr_options.len() - 1].1, 116292);
        assert_eq!(result.lastpage_up, 0);
        assert_eq!(result.last_page, 0);
        assert_eq!(result.page, -2);

        //beyond last page (almost the same as last page: 1 fewer rows since all from before and none from equal and after)
        let resp = test::TestRequest::get()
            .uri(r#"/lsj/query?n=101&idprefix=test1&x=0.795795025371805&requestTime=1637859894040&page=0&mode=context&query={%22regex%22:%220%22,%22lexicon%22:%22lsj%22,%22tag_id%22:%220%22,%22root_id%22:%220%22,%22w%22:%22%CF%89%CF%89%CF%89%22}"#)
            .send_request(&mut app).await;
        let result: QueryResponse = serde_json::from_str( resp.response().body().as_str() ).unwrap();
        assert_eq!(result.arr_options[0].1, 116495);
        assert_eq!(result.arr_options[result.arr_options.len() - 1].1, 116596);
        assert_eq!(result.lastpage_up, 0);
        assert_eq!(result.last_page, 1);
        assert_eq!(result.page, 0);

        //query μ
        let resp = test::TestRequest::get()
            .uri(r#"/lsj/query?n=101&idprefix=test1&x=0.795795025371805&requestTime=1637859894040&page=0&mode=context&query={%22regex%22:%220%22,%22lexicon%22:%22lsj%22,%22tag_id%22:%220%22,%22root_id%22:%220%22,%22w%22:%22%CE%BC%22}"#)
            .send_request(&mut app).await;
        
        let result: QueryResponse = serde_json::from_str( resp.response().body().as_str() ).unwrap();
        assert_eq!(result.arr_options[0].1, 64416);
        assert_eq!(result.arr_options[result.arr_options.len() - 1].1, 64617);
        assert_eq!(result.lastpage_up, 0);
        assert_eq!(result.last_page, 0);
        assert_eq!(result.page, 0);

        //query μ page -1
        let resp = test::TestRequest::get()
            .uri(r#"/lsj/query?n=101&idprefix=test1&x=0.795795025371805&requestTime=1637859894040&page=-1&mode=context&query={%22regex%22:%220%22,%22lexicon%22:%22lsj%22,%22tag_id%22:%220%22,%22root_id%22:%220%22,%22w%22:%22%CE%BC%22}"#)
            .send_request(&mut app).await;
        
        let result: QueryResponse = serde_json::from_str( resp.response().body().as_str() ).unwrap();
        assert_eq!(result.arr_options[0].1, 64415);
        assert_eq!(result.arr_options[result.arr_options.len() - 1].1, 64315);
        assert_eq!(result.lastpage_up, 0);
        assert_eq!(result.last_page, 0);
        assert_eq!(result.page, -1);

        //query μ page 1
        let resp = test::TestRequest::get()
            .uri(r#"/lsj/query?n=101&idprefix=test1&x=0.795795025371805&requestTime=1637859894040&page=1&mode=context&query={%22regex%22:%220%22,%22lexicon%22:%22lsj%22,%22tag_id%22:%220%22,%22root_id%22:%220%22,%22w%22:%22%CE%BC%22}"#)
            .send_request(&mut app).await;
        
        let result: QueryResponse = serde_json::from_str( resp.response().body().as_str() ).unwrap();
        assert_eq!(result.arr_options[0].1, 64618);
        assert_eq!(result.arr_options[result.arr_options.len() - 1].1, 64718);
        assert_eq!(result.lastpage_up, 0);
        assert_eq!(result.last_page, 0);
        assert_eq!(result.page, 1);

        //query αβ: near beginning so we get lastpage_up == 1
        let resp = test::TestRequest::get()
            .uri(r#"/lsj/query?n=101&idprefix=test1&x=0.795795025371805&requestTime=1637859894040&page=0&mode=context&query={%22regex%22:%220%22,%22lexicon%22:%22lsj%22,%22tag_id%22:%220%22,%22root_id%22:%220%22,%22w%22:%22%CE%B1%CE%B2%22}"#)
            .send_request(&mut app).await;
        
        let result: QueryResponse = serde_json::from_str( resp.response().body().as_str() ).unwrap();
        assert_eq!(result.arr_options[0].1, 1);
        assert_eq!(result.arr_options[result.arr_options.len() - 1].1, 135);
        assert_eq!(result.lastpage_up, 1);
        assert_eq!(result.last_page, 0);
        assert_eq!(result.page, 0);

        //query ωσσ: near end so we get lastpage == 1
        let resp = test::TestRequest::get()
            .uri(r#"/lsj/query?n=101&idprefix=test1&x=0.795795025371805&requestTime=1637859894040&page=0&mode=context&query={%22regex%22:%220%22,%22lexicon%22:%22lsj%22,%22tag_id%22:%220%22,%22root_id%22:%220%22,%22w%22:%22%CF%89%CF%83%CF%83%22}"#)
            .send_request(&mut app).await;
        
        let result: QueryResponse = serde_json::from_str( resp.response().body().as_str() ).unwrap();
        assert_eq!(result.arr_options[0].1, 116411);
        assert_eq!(result.arr_options[result.arr_options.len() - 1].1, 116596);
        assert_eq!(result.lastpage_up, 0);
        assert_eq!(result.last_page, 1);
        assert_eq!(result.page, 0);

        //DefResponse
        let resp = test::TestRequest::get()
            .uri(r#"/lsj/item?id=110628&lexicon=lsj&skipcache=0&addwordlinks=0&x=0.7049151126608002"#)
            .send_request(&mut app).await;
        let result: DefResponse = serde_json::from_str( resp.response().body().as_str() ).unwrap();
        assert_eq!(result.word_id, 110628);
        
    }

    /* other tests
    to fix: don't set last page for opposite direction: none
    check page <> 0 near top and bottom and check that last page is set.
    */
}


