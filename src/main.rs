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
extern crate unicode_normalization;
use unicode_normalization::UnicodeNormalization;

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


extern crate chrono;
use chrono::prelude::*;
use std::time::Duration;

mod db;
use crate::db::*;
use serde::{Deserialize, Serialize};

use std::time::{SystemTime, UNIX_EPOCH};

/*
{"error":"","wtprefix":"test1","nocache":"1","container":"test1Container","requestTime":"1635643672625","selectId":"32","page":"0","lastPage":"0","lastPageUp":"1","scroll":"32","query":"","arrOptions":[{"i":1,"r":["Α α",1,0]},{"i":2,"r":["ἀ-",2,0]},{"i":3,"r":["ἀ-",3,0]},{"i":4,"r":["ἆ",4,0]}...
*/

//https://stackoverflow.com/questions/64348528/how-can-i-pass-multi-variable-by-actix-web-appdata
//https://doc.rust-lang.org/rust-by-example/generics/new_types.html
#[derive(Clone)]
struct SqliteUpdatePool (SqlitePool);

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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SynopsisSaverRequest {
    pub advisor: String,
    pub day: String,
    pub sname: String,
    pub number: String,
    pub person: String,
    pub pp: String,
    pub ptccase: String,
    pub ptcgender: String,
    pub ptcnumber: String,
    pub r: Vec<String>,
    pub verb: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SynopsisResultRequest {
    pub id:u32,
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
        //remove any diacritics and make lowercase
        //println!("1: {}",query_params.w);
        let q = query_params.w.nfd().filter(|x| !unicode_normalization::char::is_combining_mark(*x) && *x != '´' && *x != '`' && *x != '῀').collect::<String>().to_lowercase();
        //println!("2: {}",q);
        get_seq_by_prefix(db, table, &q).await.map_err(map_sqlx_error)?
    }
    else {
        let decoded_word = percent_decode_str(query_params.wordid.as_ref().unwrap()).decode_utf8().map_err(map_utf8_error)?;
        get_seq_by_word(db, table, &decoded_word).await.map_err(map_sqlx_error)?
    };

    let mut before_rows = vec![];
    let mut after_rows = vec![];
    if info.page <= 0 {
        
        before_rows = get_before(db, table, seq, info.page, info.n).await.map_err(map_sqlx_error)?;
        if info.page == 0 { //only reverse if page 0. if < 0, each row is inserted under top of container one-by-one in order
            before_rows.reverse();
        }
    }
    if info.page >= 0 {
        after_rows = get_equal_and_after(db, table, seq, info.page, info.n).await.map_err(map_sqlx_error)?;
    }

    //only check page 0 or page less than 0
    let vlast_page_up = if before_rows.len() < info.n as usize && info.page <= 0 { 1 } else { 0 };
    //only check page 0 or page greater than 0
    let vlast_page = if after_rows.len() < info.n as usize && info.page >= 0 { 1 } else { 0 };

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
        scroll: if query_params.w.is_empty() && info.page == 0 && seq == 1 { "top".to_string() } else { "".to_string() }, //scroll really only needs to return top
        query: query_params.w.to_owned(),
        arr_options: result_rows_stripped
    };

    Ok(HttpResponse::Ok().json(res))
}

fn get_user_agent(req: &HttpRequest) -> Option<&str> {
    req.headers().get("user-agent")?.to_str().ok()
}

//http://127.0.0.1:8088/wordservjson.php?id=110628&lexicon=lsj&skipcache=0&addwordlinks=0&x=0.7049151126608002
//{"principalParts":"","def":"...","defName":"","word":"γεοῦχος","unaccentedWord":"γεουχοσ","lemma":"γεοῦχος","requestTime":0,"status":"0","lexicon":"lsj","word_id":"22045","wordid":"γεουχοσ","method":"setWord"}

#[allow(clippy::eval_order_dependence)]
async fn philologus_defs((info, req): (web::Query<DefRequest>, HttpRequest)) -> Result<HttpResponse, AWError> {
    let db = req.app_data::<SqlitePool>().unwrap();
    let db2 = req.app_data::<SqliteUpdatePool>().unwrap();
    
    let table = match info.lexicon.as_str() {
        "ls" => "ZLATIN",
        "slater" => "ZSLATER",
        _ => "ZGREEK"
    };

    let def_row = if info.wordid.is_some() {
        let decoded_word = percent_decode_str( info.wordid.as_ref().unwrap() ).decode_utf8().map_err(map_utf8_error)?;
        get_def_by_word(db, table, &decoded_word ).await.map_err(map_sqlx_error)?
    }
    else if info.id.is_some() {
        get_def_by_seq(db, table, info.id.unwrap() ).await.map_err(map_sqlx_error)?
    }
    else {
        return Err(PhilologusError { code: StatusCode::INTERNAL_SERVER_ERROR, name: "philologus_defs error: word and id are both empty".to_string(), error: "philologus_defs error: word and id are both empty".to_string() }.into() )
    };


    let lex = match info.lexicon.as_str() { "ls" => 1, "slater" => 2, _ => 0 };
    let time_stamp = SystemTime::now().duration_since(UNIX_EPOCH);
    let time_stamp_ms = if time_stamp.is_ok() { time_stamp.unwrap().as_millis() } else { 0 };
    let user_agent = get_user_agent(&req).unwrap_or("");
    //https://stackoverflow.com/questions/66989780/how-to-retrieve-the-ip-address-of-the-client-from-httprequest-in-actix-web
    let ip = if req.peer_addr().is_some() { req.peer_addr().unwrap().ip().to_string() } else { "".to_string() };
    let _ = insert_log(&db2.0, time_stamp_ms, lex, def_row.seq, ip.as_str(), user_agent).await;

    let def = add_bibl_links(&def_row.def);
    
    let res = DefResponse {
        principal_parts: None,
        def,
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

fn add_bibl_links(def:&str) -> String {

    let re = Regex::new(r#"biblink="(?P<l>[^"]*)""#).unwrap();
    let def_with_links = re.replace_all(def, r#"href="http://www.perseus.tufts.edu/hopper/text.jsp?doc=$l&amp;lang=original" target="_blank""#);

    def_with_links.to_string()
}

async fn index(_req: HttpRequest) -> Result<NamedFile> {
    let path: PathBuf = "static/index.html".parse().unwrap();
    Ok(NamedFile::open(path)?)
}

async fn health_check(_req: HttpRequest) -> Result<HttpResponse, AWError> {
    Ok(HttpResponse::Ok().finish()) //send 200 with empty body
}

#[allow(clippy::eval_order_dependence)]
async fn synopsis_list(req: HttpRequest) -> Result<HttpResponse, AWError> {
    let db2 = req.app_data::<SqliteUpdatePool>().unwrap();

    let list = get_synopsis_list(&db2.0).await.map_err(map_sqlx_error)?;

    let mut res = String::from(r#"<!DOCTYPE html>
    <html>
    <head>
    <meta charset="UTF-8">
    </head>
    <body><table>"#);
    for l in list {
        let d = UNIX_EPOCH + Duration::from_millis(l.1.try_into().unwrap());
        let datetime = DateTime::<Local>::from(d);
        let timestamp_str = datetime.format("%Y-%m-%d %H:%M:%S").to_string();

        res.push_str(format!("<tr><td><a href='synopsisresult?sid={}'>{}</a></td><td>{}</td><td>{}</td><td>{}</td></tr>", l.0, timestamp_str, l.2, l.3,l.4).as_str());
    }
    res.push_str("</table></body></html>");

    Ok(HttpResponse::Ok()
            .content_type("text/html")
            .body(res))
}

#[allow(clippy::eval_order_dependence)]
async fn synopsis_result((info, req):(web::Query<SynopsisResultRequest>, HttpRequest)) -> Result<HttpResponse, AWError> {
    let db2 = req.app_data::<SqliteUpdatePool>().unwrap();

    let list = get_synopsis_result(&db2.0, info.id).await.map_err(map_sqlx_error)?;

    let mut res = String::from(r#"<!DOCTYPE html>
    <html>
    <head>
    <meta charset="UTF-8">
    </head>
    <body><table>"#);
    for l in list {
        let d = UNIX_EPOCH + Duration::from_millis(l.1.try_into().unwrap());
        let datetime = DateTime::<Local>::from(d);
        let timestamp_str = datetime.format("%Y-%m-%d %H:%M:%S").to_string();

        res.push_str(format!("<tr><td><a href='synopsisresult?sid={}'>{}</a></td><td>{}</td><td>{}</td><td>{}</td></tr>", l.0, timestamp_str, l.2, l.3,l.4).as_str());
    }
    res.push_str("</table></body></html>");

    Ok(HttpResponse::Ok()
            .content_type("text/html")
            .body(res))
}

#[allow(clippy::eval_order_dependence)]
async fn synopsis_saver((info, req): (web::Json<SynopsisSaverRequest>, HttpRequest)) -> Result<HttpResponse, AWError> {
    let db2 = req.app_data::<SqliteUpdatePool>().unwrap();

    let time_stamp = SystemTime::now().duration_since(UNIX_EPOCH);
    let time_stamp_ms = if time_stamp.is_ok() { time_stamp.unwrap().as_millis() } else { 0 };
    let user_agent = get_user_agent(&req).unwrap_or("");
    //https://stackoverflow.com/questions/66989780/how-to-retrieve-the-ip-address-of-the-client-from-httprequest-in-actix-web
    let ip = if req.peer_addr().is_some() { req.peer_addr().unwrap().ip().to_string() } else { "".to_string() };

    let _ = insert_synopsis(&db2.0, &info.into_inner(), time_stamp_ms, ip.as_str(), user_agent).await.map_err(map_sqlx_error)?;
    
    //Ok(HttpResponse::Ok().finish())
    //let res = 1;
    Ok(HttpResponse::Ok().json(1))
}

#[allow(clippy::eval_order_dependence)]
async fn synopsis(_req: HttpRequest) -> Result<HttpResponse, AWError> {
    let mut template = include_str!("synopsis.html").to_string();

    let mut rows = String::from("");
    let mut count = 0;
    let rowlabels = vec!["Present Indicative", "Future Indicative", "Imperfect Indicative", "Aorist Indicative", "Perfect Indicative", "Pluperfect Indicative", "Present Subjunctive", "Aorist Subjunctive", "Present Optative", "Future Optative", "Aorist Optative","Present Imperative", "Aorist Imperative", "Present Infinitive", "Future Infinitive", "Aorist Infinitive", "Perfect Infinitive", "Present Participle", "Future Participle", "Aorist Participle", "Perfect Participle"];
    let voices = vec!["Active", "Middle", "Passive"];
    for l in rowlabels {
        rows.push_str(format!(r#"<tr class="{}"><td>{}</td>"#, l.to_lowercase(), l).as_str());
        for v in &voices {
            rows.push_str(format!(
            r#"<td class="formcell {}">
                <div class="formcellInner">
                <input type="text" id="gkform{}" class="gkinput formcellinput" spellcheck="false" autocapitalize="off" autocomplete="off"/>
                </div>
            </td>"#, 
            v.to_lowercase(), count).as_str());
            count += 1;
        }
        rows.push_str("</tr>");
    }

    template = template.replacen("%rows%", &rows, 1);

    Ok(HttpResponse::Ok()
            .content_type("text/html")
            .body(template))
}

async fn hc(_req: HttpRequest) -> Result<HttpResponse, AWError> {
    Ok(HttpResponse::Ok().finish())
}

// fn json_error_handler(error: JsonPayloadError, _: &HttpRequest) -> actix_web::Error {
//     match error {
//         JsonPayloadError::Overflow => ServiceError::PayloadTooLarge.into(),
//         _ => ServiceError::BadRequest(error.to_string()).into(),
//     }
// }

#[actix_web::main]
async fn main() -> io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    //e.g. export PHILOLOGUS_DB_PATH=sqlite://philolog_us_local.sqlite?mode=ro
    //e.g. export PHILOLOGUS_LOG_DB_PATH=sqlite://updatedb.sqlite?mode=rwc
    let db_path = std::env::var("PHILOLOGUS_DB_PATH")
                   .unwrap_or_else(|_| panic!("Environment variable for sqlite path not set: PHILOLOGUS_DB_PATH."));
    let db_pool = SqlitePool::connect(&db_path).await.expect("Could not connect to db.");

    let db_log_path = std::env::var("PHILOLOGUS_LOG_DB_PATH")
                    .unwrap_or_else(|_| panic!("Environment variable for sqlite log path not set: PHILOLOGUS_LOG_DB_PATH."));

    //https://gitanswer.com/sqlx-how-to-create-the-sqlite-database-on-application-startup-if-it-does-not-already-exist-rust-833366308
    /*
    if !sqlx::Sqlite::database_exists(&db_log_path).await? {
        sqlx::Sqlite::create_database(&db_log_path).await?;
    }
    */
    let db_log_pool = SqliteUpdatePool(SqlitePool::connect(&db_log_path).await.expect("Could not connect to update db."));
    let query = "CREATE TABLE IF NOT EXISTS log (id integer primary key autoincrement, accessed integer, lexicon integer, word integer, ip text, agent text);";
    let _ = sqlx::query(query).execute(&db_log_pool.0).await;

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
            .app_data(db_log_pool.clone())
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            // .app_data(
            //     web::JsonConfig::default()
            //         .error_handler(json_error_handler)
            //         .limit(262_144),
            // )
            .app_data(web::PayloadConfig::default().limit(262_144))
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
            .service(
                web::resource("/healthzzz")
                    .route(web::get().to(health_check)),
            )
            .service(
                web::resource("/synopsisresult")
                    .route(web::get().to(synopsis_result)),
            )
            .service(
                web::resource("/synopsislist")
                    .route(web::get().to(synopsis_list)),
            )
            .service(
                web::resource("/synopsissaver")
                    .route(web::post().to(synopsis_saver)),
            )
            .service(
                web::resource("/synopsis")
                    .route(web::get().to(synopsis)),
            )
            .service(
                web::resource("/hc.php")
                    .route(web::get().to(hc)),
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
            error: self.name.to_string(),
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

    #[test]
    async fn json_test() {
        let s = r#"{"pp":"ἵστημι, στήσω, ἔστησα / ἔστην, ἕστηκα, ἕσταμαι, ἐστάθην","day":22,"verb":"αα","person":"2nd","number":"sing","ptccase":"dat","ptcgender":"fem","ptcnumber":"","sname":"name","advisor":"advisor","r":["ββ","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","γγ"]}"#;
        let r: SynopsisSaverRequest = serde_json::from_str(&s).unwrap();
        assert_eq!(r.verb, "αα".to_string());
        assert_eq!(r.r[62], "γγ".to_string());
    }

    #[test]
    async fn test_unicode_strip_diacritics_and_lowercase() {
        let a = "ἄέώΏ".nfd().filter(|x| !unicode_normalization::char::is_combining_mark(*x)).collect::<String>().to_lowercase();
        assert_eq!(a, "αεωω");
    }

    #[test]
    async fn test_add_links() {
        let a = r#"blah biblink="Perseus:abo:tlg,0059,005:405c"> blahblah biblink="Perseus:abo:tlg,4083,001:641:61">"#;
        let b = add_bibl_links(a);
        assert_eq!(b, r#"blah href="http://www.perseus.tufts.edu/hopper/text.jsp?doc=Perseus:abo:tlg,0059,005:405c&amp;lang=original" target="_blank"> blahblah href="http://www.perseus.tufts.edu/hopper/text.jsp?doc=Perseus:abo:tlg,4083,001:641:61&amp;lang=original" target="_blank">"#);
    }

    #[actix_web::test]
    async fn test_query_paging() {
        let db_path = std::env::var("PHILOLOGUS_DB_PATH")
                   .unwrap_or_else(|_| panic!("Environment variable for sqlite path not set: PHILOLOGUS_DB_PATH."));

        let db_pool = SqlitePool::connect(&db_path).await.expect("Could not connect to db.");
        let db_pool2 = SqliteUpdatePool(SqlitePool::connect("sqlite://updatedb.sqlite").await.expect("Could not connect to update db."));

        let mut app = test::init_service(
            App::new()
            .app_data(db_pool.clone())
            .app_data(db_pool2.clone())
            .service(
                web::resource("/{lex}/query")
                    .route(web::get().to(philologus_words))
            )
            .service(
                web::resource("/healthzzz")
                    .route(web::get().to(health_check)),
            )
            .service(
                web::resource("/hc.php")
                    .route(web::get().to(hc)),
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
        let result: QueryResponse = test::read_body_json(resp).await;
        //println!("res: {:?}", result);
        assert_eq!(result.arr_options.len(), 202);

        //empty query returns seq 1 for first row
        let resp = test::TestRequest::get()
            .uri(r#"/lsj/query?n=101&idprefix=test1&x=0.795795025371805&requestTime=1637859894040&page=0&mode=context&query={%22regex%22:%220%22,%22lexicon%22:%22lsj%22,%22tag_id%22:%220%22,%22root_id%22:%220%22,%22w%22:%22%22}"#)
            .send_request(&mut app).await;
        let result: QueryResponse = test::read_body_json(resp).await;
        
        assert_eq!(result.arr_options[0].1, 1);
        assert_eq!(result.arr_options[result.arr_options.len() - 1].1, 101);
        assert_eq!(result.lastpage_up, 1);
        assert_eq!(result.last_page, 0);
        assert_eq!(result.page, 0);

        //page 1
        let resp = test::TestRequest::get()
            .uri(r#"/lsj/query?n=101&idprefix=test1&x=0.795795025371805&requestTime=1637859894040&page=1&mode=context&query={%22regex%22:%220%22,%22lexicon%22:%22lsj%22,%22tag_id%22:%220%22,%22root_id%22:%220%22,%22w%22:%22%22}"#)
            .send_request(&mut app).await;
        let result: QueryResponse = test::read_body_json(resp).await;
        
        assert_eq!(result.arr_options[0].1, 102);
        assert_eq!(result.arr_options[result.arr_options.len() - 1].1, 202);
        assert_eq!(result.lastpage_up, 0);
        assert_eq!(result.last_page, 0);
        assert_eq!(result.page, 1);
        //page 2
        let resp = test::TestRequest::get()
            .uri(r#"/lsj/query?n=101&idprefix=test1&x=0.795795025371805&requestTime=1637859894040&page=2&mode=context&query={%22regex%22:%220%22,%22lexicon%22:%22lsj%22,%22tag_id%22:%220%22,%22root_id%22:%220%22,%22w%22:%22%22}"#)
            .send_request(&mut app).await;
        let result: QueryResponse = test::read_body_json(resp).await;
        
        assert_eq!(result.arr_options[0].1, 203);
        assert_eq!(result.arr_options[result.arr_options.len() - 1].1, 303);
        assert_eq!(result.lastpage_up, 0);
        assert_eq!(result.last_page, 0);
        assert_eq!(result.page, 2);

        //alpha query returns seq 1 for first row (exactly like empty query)
        let resp = test::TestRequest::get()
            .uri(r#"/lsj/query?n=101&idprefix=test1&x=0.795795025371805&requestTime=1637859894040&page=0&mode=context&query={%22regex%22:%220%22,%22lexicon%22:%22lsj%22,%22tag_id%22:%220%22,%22root_id%22:%220%22,%22w%22:%22%CE%B1%22}"#)
            .send_request(&mut app).await;
        let result: QueryResponse = test::read_body_json(resp).await;
        
        assert_eq!(result.arr_options[0].1, 1);
        assert_eq!(result.arr_options[result.arr_options.len() - 1].1, 101);
        assert_eq!(result.lastpage_up, 1);
        assert_eq!(result.last_page, 0);
        assert_eq!(result.page, 0);


        //last page
        let resp = test::TestRequest::get()
            .uri(r#"/lsj/query?n=101&idprefix=test1&x=0.795795025371805&requestTime=1637859894040&page=0&mode=context&query={%22regex%22:%220%22,%22lexicon%22:%22lsj%22,%22tag_id%22:%220%22,%22root_id%22:%220%22,%22w%22:%22%CF%89%CF%89%22}"#)
            .send_request(&mut app).await;
        let result: QueryResponse = test::read_body_json(resp).await;
        
        assert_eq!(result.arr_options[0].1, 116494);
        assert_eq!(result.arr_options[result.arr_options.len() - 1].1, 116596);
        assert_eq!(result.lastpage_up, 0);
        assert_eq!(result.last_page, 1);
        assert_eq!(result.page, 0);

        //last page - 1 (remember these pages are delivered in reverse order when page < 0)
        let resp = test::TestRequest::get()
            .uri(r#"/lsj/query?n=101&idprefix=test1&x=0.795795025371805&requestTime=1637859894040&page=-1&mode=context&query={%22regex%22:%220%22,%22lexicon%22:%22lsj%22,%22tag_id%22:%220%22,%22root_id%22:%220%22,%22w%22:%22%CF%89%CF%89%22}"#)
            .send_request(&mut app).await;
        let result: QueryResponse = test::read_body_json(resp).await;
        
        assert_eq!(result.arr_options[0].1, 116493);
        assert_eq!(result.arr_options[result.arr_options.len() - 1].1, 116393);
        assert!(result.arr_options[result.arr_options.len() - 1].1 < result.arr_options[0].1);
        assert_eq!(result.lastpage_up, 0);
        assert_eq!(result.last_page, 0);
        assert_eq!(result.page, -1);

        //last page - 2 (remember these pages are delivered in reverse order when page < 0)
        let resp = test::TestRequest::get()
            .uri(r#"/lsj/query?n=101&idprefix=test1&x=0.795795025371805&requestTime=1637859894040&page=-2&mode=context&query={%22regex%22:%220%22,%22lexicon%22:%22lsj%22,%22tag_id%22:%220%22,%22root_id%22:%220%22,%22w%22:%22%CF%89%CF%89%22}"#)
            .send_request(&mut app).await;
        let result: QueryResponse = test::read_body_json(resp).await;
        
        assert_eq!(result.arr_options[0].1, 116392);
        assert_eq!(result.arr_options[result.arr_options.len() - 1].1, 116292);
        assert_eq!(result.lastpage_up, 0);
        assert_eq!(result.last_page, 0);
        assert_eq!(result.page, -2);

        //beyond last page (almost the same as last page: 1 fewer rows since all from before and none from equal and after)
        let resp = test::TestRequest::get()
            .uri(r#"/lsj/query?n=101&idprefix=test1&x=0.795795025371805&requestTime=1637859894040&page=0&mode=context&query={%22regex%22:%220%22,%22lexicon%22:%22lsj%22,%22tag_id%22:%220%22,%22root_id%22:%220%22,%22w%22:%22%CF%89%CF%89%CF%89%22}"#)
            .send_request(&mut app).await;
        let result: QueryResponse = test::read_body_json(resp).await;
        
        assert_eq!(result.arr_options[0].1, 116495);
        assert_eq!(result.arr_options[result.arr_options.len() - 1].1, 116596);
        assert_eq!(result.lastpage_up, 0);
        assert_eq!(result.last_page, 1);
        assert_eq!(result.page, 0);

        //query μ
        let resp = test::TestRequest::get()
            .uri(r#"/lsj/query?n=101&idprefix=test1&x=0.795795025371805&requestTime=1637859894040&page=0&mode=context&query={%22regex%22:%220%22,%22lexicon%22:%22lsj%22,%22tag_id%22:%220%22,%22root_id%22:%220%22,%22w%22:%22%CE%BC%22}"#)
            .send_request(&mut app).await;
        let result: QueryResponse = test::read_body_json(resp).await;
        
        assert_eq!(result.arr_options[0].1, 64416);
        assert_eq!(result.arr_options[result.arr_options.len() - 1].1, 64617);
        assert_eq!(result.lastpage_up, 0);
        assert_eq!(result.last_page, 0);
        assert_eq!(result.page, 0);

        //query μ page -1
        let resp = test::TestRequest::get()
            .uri(r#"/lsj/query?n=101&idprefix=test1&x=0.795795025371805&requestTime=1637859894040&page=-1&mode=context&query={%22regex%22:%220%22,%22lexicon%22:%22lsj%22,%22tag_id%22:%220%22,%22root_id%22:%220%22,%22w%22:%22%CE%BC%22}"#)
            .send_request(&mut app).await;
        let result: QueryResponse = test::read_body_json(resp).await;
        
        assert_eq!(result.arr_options[0].1, 64415);
        assert_eq!(result.arr_options[result.arr_options.len() - 1].1, 64315);
        assert_eq!(result.lastpage_up, 0);
        assert_eq!(result.last_page, 0);
        assert_eq!(result.page, -1);

        //query μ page 1
        let resp = test::TestRequest::get()
            .uri(r#"/lsj/query?n=101&idprefix=test1&x=0.795795025371805&requestTime=1637859894040&page=1&mode=context&query={%22regex%22:%220%22,%22lexicon%22:%22lsj%22,%22tag_id%22:%220%22,%22root_id%22:%220%22,%22w%22:%22%CE%BC%22}"#)
            .send_request(&mut app).await;
        let result: QueryResponse = test::read_body_json(resp).await;
        
        assert_eq!(result.arr_options[0].1, 64618);
        assert_eq!(result.arr_options[result.arr_options.len() - 1].1, 64718);
        assert_eq!(result.lastpage_up, 0);
        assert_eq!(result.last_page, 0);
        assert_eq!(result.page, 1);

        //query αβ: near beginning so we get lastpage_up == 1
        let resp = test::TestRequest::get()
            .uri(r#"/lsj/query?n=101&idprefix=test1&x=0.795795025371805&requestTime=1637859894040&page=0&mode=context&query={%22regex%22:%220%22,%22lexicon%22:%22lsj%22,%22tag_id%22:%220%22,%22root_id%22:%220%22,%22w%22:%22%CE%B1%CE%B2%22}"#)
            .send_request(&mut app).await;
        let result: QueryResponse = test::read_body_json(resp).await;
        
        assert_eq!(result.arr_options[0].1, 1);
        assert_eq!(result.arr_options[result.arr_options.len() - 1].1, 135);
        assert_eq!(result.lastpage_up, 1);
        assert_eq!(result.last_page, 0);
        assert_eq!(result.page, 0);

        //query ωσσ: near end so we get lastpage == 1
        let resp = test::TestRequest::get()
            .uri(r#"/lsj/query?n=101&idprefix=test1&x=0.795795025371805&requestTime=1637859894040&page=0&mode=context&query={%22regex%22:%220%22,%22lexicon%22:%22lsj%22,%22tag_id%22:%220%22,%22root_id%22:%220%22,%22w%22:%22%CF%89%CF%83%CF%83%22}"#)
            .send_request(&mut app).await;
        let result: QueryResponse = test::read_body_json(resp).await;
        
        assert_eq!(result.arr_options[0].1, 116411);
        assert_eq!(result.arr_options[result.arr_options.len() - 1].1, 116596);
        assert_eq!(result.lastpage_up, 0);
        assert_eq!(result.last_page, 1);
        assert_eq!(result.page, 0);


        //query ως: page 1 passing end
        let resp = test::TestRequest::get()
            .uri(r#"/lsj/query?n=101&idprefix=test1&x=0.795795025371805&requestTime=1637859894040&page=1&mode=context&query={%22regex%22:%220%22,%22lexicon%22:%22lsj%22,%22tag_id%22:%220%22,%22root_id%22:%220%22,%22w%22:%22%CF%89%CF%82%22}"#)
            .send_request(&mut app).await;
        let result: QueryResponse = test::read_body_json(resp).await;
        
        assert_eq!(result.arr_options[0].1, 116593);
        assert_eq!(result.arr_options[result.arr_options.len() - 1].1, 116596);
        assert_eq!(result.lastpage_up, 0);
        assert_eq!(result.last_page, 1);
        assert_eq!(result.page, 1);

        //query αβλε: page -1 passing beginning
        let resp = test::TestRequest::get()
            .uri(r#"/lsj/query?n=101&idprefix=test1&x=0.795795025371805&requestTime=1637859894040&page=-1&mode=context&query={%22regex%22:%220%22,%22lexicon%22:%22lsj%22,%22tag_id%22:%220%22,%22root_id%22:%220%22,%22w%22:%22%CE%B1%CE%B2%CE%BB%CE%B5%22}"#)
            .send_request(&mut app).await;

        let result: QueryResponse = test::read_body_json(resp).await;
        
        assert_eq!(result.arr_options[0].1, 8);
        assert_eq!(result.arr_options[result.arr_options.len() - 1].1, 1);
        assert_eq!(result.lastpage_up, 1);
        assert_eq!(result.last_page, 0);
        assert_eq!(result.page, -1);

        //health check
        let resp = test::TestRequest::get()
            .uri(r#"/healthzzz"#)
            .send_request(&mut app).await;
        assert!(&resp.status().is_success());

        //hoplite challenge
        let resp = test::TestRequest::get()
            .uri(r#"/hc.php"#)
            .send_request(&mut app).await;
        assert!(&resp.status().is_success());
        
        //DefResponse
        let resp = test::TestRequest::get()
            .uri(r#"/lsj/item?id=110628&lexicon=lsj&skipcache=0&addwordlinks=0&x=0.7049151126608002"#)
            .send_request(&mut app).await;
        let result: DefResponse = test::read_body_json(resp).await;
        
        assert_eq!(result.word_id, 110628);

        //DefResponse: both word and id are empty
        let resp = test::TestRequest::get()
        .uri(r#"/lsj/item?lexicon=lsj&skipcache=0&addwordlinks=0&x=0.7049151126608002"#)
        .send_request(&mut app).await;
        assert_eq!(resp.status(), 500);       
    }
}
