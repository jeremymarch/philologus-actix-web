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
use tracing_subscriber::fmt::writer::MakeWriterExt;
//use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_actix_web::TracingLogger;
//use tracing::info;

use actix_files as fs;
use actix_files::NamedFile;
use actix_web::{
    http::StatusCode, middleware, web, App, Error as AWError, HttpRequest, HttpResponse,
    HttpServer, ResponseError, Result,
};

use chrono::prelude::*;
use percent_encoding::percent_decode_str;
use regex::Regex;
use sqlx::SqlitePool;
use std::io;
use std::path::PathBuf;
use std::time::Duration;
use thiserror::Error;
use unicode_normalization::UnicodeNormalization;

mod db;
use crate::db::*;
use serde::{Deserialize, Serialize};

use std::time::{SystemTime, UNIX_EPOCH};

use tantivy::collector::{Count, TopDocs};
use tantivy::query::QueryParser;
use tantivy::{Index, ReloadPolicy};

/*
{"error":"","wtprefix":"test1","nocache":"1","container":"test1Container","requestTime":"1635643672625","selectId":"32","page":"0","lastPage":"0","lastPageUp":"1","scroll":"32","query":"","arrOptions":[{"i":1,"r":["Α α",1,0]},{"i":2,"r":["ἀ-",2,0]},{"i":3,"r":["ἀ-",3,0]},{"i":4,"r":["ἆ",4,0]}...
*/

//https://stackoverflow.com/questions/64348528/how-can-i-pass-multi-variable-by-actix-web-appdata
//https://doc.rust-lang.org/rust-by-example/generics/new_types.html
#[derive(Clone)]
struct SqliteUpdatePool(SqlitePool);

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
    query: String,
    #[serde(rename(serialize = "arrOptions"), rename(deserialize = "arrOptions"))]
    arr_options: Vec<(String, u32)>,
}

struct QueryResult {
    seq: u32,
    vlast_page_up: u8,
    vlast_page: u8,
    rows: Vec<(String, u32)>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct DefResponse {
    #[serde(
        rename(serialize = "principalParts"),
        rename(deserialize = "principalParts")
    )]
    principal_parts: Option<String>,
    def: String,
    #[serde(rename(serialize = "defName"), rename(deserialize = "defName"))]
    def_name: Option<String>,
    word: String,
    #[serde(
        rename(serialize = "unaccentedWord"),
        rename(deserialize = "unaccentedWord")
    )]
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
pub struct FullTextQueryRequest {
    pub q: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct LexEntry {
    id: u64,
    lemma: String,
    lex: String,
    def: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct FullTextResponse {
    ftresults: Vec<LexEntry>,
    error: String,
    count: usize,
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
    pub query: String, //WordQuery,
    pub lex: Option<String>,
}

#[derive(Deserialize)]
pub struct WordQuery {
    pub regex: i32,
    pub lexicon: String,
    pub tag_id: i32,
    pub root_id: i32,
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
    pub unit: String,
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
    pub id: u32,
}

//http://127.0.0.1:8088/philwords?n=101&idprefix=test1&x=0.1627681205837177&requestTime=1635643672625&page=0&mode=context&query={%22regex%22:%220%22,%22lexicon%22:%22lsj%22,%22tag_id%22:%220%22,%22root_id%22:%220%22,%22wordid%22:%22%CE%B1%CE%B1%CF%84%CE%BF%CF%832%22,%22w%22:%22%22}

async fn full_text_query(
    (info, req): (web::Query<FullTextQueryRequest>, HttpRequest),
) -> Result<HttpResponse, AWError> {
    let db = req.app_data::<SqlitePool>().unwrap();
    let index = req.app_data::<tantivy::Index>().unwrap();

    // let word_id_field = index.schema().get_field("word_id").unwrap();
    // let lemma_field = index.schema().get_field("lemma").unwrap();
    let lexicon_field = index.schema().get_field("lexicon").unwrap();
    let definition_field = index.schema().get_field("definition").unwrap();

    let reader = index
        .reader_builder()
        .reload_policy(ReloadPolicy::OnCommit)
        .try_into()
        .unwrap();

    let searcher = reader.searcher();
    let query_parser = QueryParser::for_index(
        index,
        //this vector contains default fields used if field is not specified in query
        vec![lexicon_field, definition_field],
    );

    let mut res = FullTextResponse {
        ftresults: vec![],
        error: String::from(""),
        count: 0,
    };

    // full-text index should be all lowercase, but use uppercase for AND and OR
    let mut ft_query = info.q.to_lowercase();
    ft_query = ft_query.replace(" and ", " AND ").replace(" or ", " OR ");

    let my_collector = (Count, TopDocs::with_limit(10));
    match query_parser.parse_query(&ft_query) {
        //"carry AND (lexicon:slater OR lexicon:lewisshort)") {
        Ok(query) => match searcher.search(&query, &my_collector) {
            Ok((count, top_docs)) => {
                for (_score, doc_address) in top_docs {
                    match searcher.doc(doc_address) {
                        Ok(retrieved_doc) => {
                            let mut word_id_value: u32 = 0;
                            let mut lexicon_value: String = String::from("");

                            for (field, field_values) in retrieved_doc.get_sorted_field_values() {
                                match index.schema().get_field_name(field) {
                                    "lexicon" => {
                                        lexicon_value =
                                            field_values[0].as_text().unwrap_or("").to_string()
                                    }
                                    "word_id" => {
                                        word_id_value = field_values[0]
                                            .as_u64()
                                            .unwrap_or(0)
                                            .try_into()
                                            .unwrap_or(0)
                                    }
                                    _ => continue,
                                }
                            }

                            // skip entry if these values aren't found
                            // this shouldn't happen
                            if word_id_value == 0 || lexicon_value.is_empty() {
                                continue;
                            }

                            let d = get_def_by_seq(db, word_id_value)
                                .await
                                .map_err(map_sqlx_error)?;

                            let entry = LexEntry {
                                id: d.seq as u64,
                                lemma: d.word,
                                lex: lexicon_value,
                                def: d.def,
                            };

                            res.ftresults.push(entry);
                        }
                        Err(e) => {
                            println!("Full-text error retrieving document: {:?}", e);
                            res.error = format!("Full-text error retrieving document: {:?}", e);
                        }
                    }
                }
                res.count = count;
            }
            Err(e) => {
                println!("Full-text error searching document: {:?}", e);
                res.error = format!("Full-text error searching document: {:?}", e);
            }
        },
        Err(e) => {
            println!("Error parsing full-text query: {:?}", e);
            res.error = format!("Error parsing full-text query: {:?}", e);
        }
    }
    Ok(HttpResponse::Ok().json(res))
}

//remove any diacritics and make lowercase
fn sanitize_query(query: &str) -> String {
    query
        .nfd()
        .filter(|x| {
            !unicode_normalization::char::is_combining_mark(*x)
                && *x != '´'
                && *x != '`'
                && *x != '῀'
        })
        .collect::<String>()
        .to_lowercase()
        .replace('ς', "σ")
}

async fn query_words(
    db: &SqlitePool,
    word_id: Option<String>,
    query: &str,
    table: &str,
    limit: u32,
    page: i32,
) -> Result<QueryResult, AWError> {
    let seq = if word_id.is_none() {
        let query = sanitize_query(query);
        get_seq_by_prefix(db, table, &query).await.unwrap()
    } else {
        let decoded_word = percent_decode_str(word_id.as_ref().unwrap())
            .decode_utf8()
            .map_err(map_utf8_error)?;
        get_seq_by_word(db, table, &decoded_word).await.unwrap()
    };

    let mut before_rows = vec![];
    let mut after_rows = vec![];
    if page <= 0 {
        before_rows = get_before(db, table, seq, page, limit).await.unwrap();
        if page == 0 {
            // only reverse if page 0. if < 0, each row is inserted under top
            // of container one-by-one in order
            before_rows.reverse();
        }
    }
    if page >= 0 {
        after_rows = get_equal_and_after(db, table, seq, page, limit)
            .await
            .unwrap();
    }

    //only check page 0 or page less than 0
    let vlast_page_up = if before_rows.len() < limit as usize && page <= 0 {
        1
    } else {
        0
    };
    //only check page 0 or page greater than 0
    let vlast_page = if after_rows.len() < limit as usize && page >= 0 {
        1
    } else {
        0
    };

    //strip any numbers from end of string
    let re = Regex::new(r"[0-9]").unwrap();
    let result_rows_stripped = [before_rows, after_rows]
        .concat()
        .into_iter()
        .map(|mut row| {
            row.0 = re.replace_all(&row.0, "").to_string();
            row
        })
        .collect();

    let res = QueryResult {
        seq: if query.is_empty() && page == 0 && word_id.is_none() {
            0
        } else {
            seq
        },
        vlast_page_up,
        vlast_page,
        rows: result_rows_stripped,
    };

    Ok(res)
}

async fn philologus_words(
    (info, req): (web::Query<QueryRequest>, HttpRequest),
) -> Result<HttpResponse, AWError> {
    let db = req.app_data::<SqlitePool>().unwrap();

    let query_params: WordQuery = serde_json::from_str(&info.query)?;

    let table = get_long_lex(query_params.lexicon.as_str());

    let query_result = query_words(
        db,
        query_params.wordid.clone(),
        &query_params.w,
        table,
        info.n,
        info.page,
    )
    .await
    .unwrap();

    let res = QueryResponse {
        select_id: query_result.seq,
        error: "".to_owned(),
        wtprefix: info.idprefix.clone(),
        nocache: if query_params.wordid.is_none() { 0 } else { 1 }, //prevents caching when queried by wordid in url
        container: format!("{}Container", info.idprefix),
        request_time: info.request_time,
        page: info.page,
        last_page: query_result.vlast_page,
        lastpage_up: query_result.vlast_page_up,
        query: query_params.w.to_owned(),
        arr_options: query_result.rows,
    };

    Ok(HttpResponse::Ok().json(res))
}

fn get_user_agent(req: &HttpRequest) -> Option<&str> {
    req.headers().get("user-agent")?.to_str().ok()
}

fn get_long_lex(lex: &str) -> &str {
    match lex {
        "ls" => "lewisshort",
        "slater" => "slater",
        _ => "lsj",
    }
}

//http://127.0.0.1:8088/wordservjson.php?id=110628&lexicon=lsj&skipcache=0&addwordlinks=0&x=0.7049151126608002
//{"principalParts":"","def":"...","defName":"","word":"γεοῦχος","unaccentedWord":"γεουχοσ","lemma":"γεοῦχος","requestTime":0,"status":"0","lexicon":"lsj","word_id":"22045","wordid":"γεουχοσ","method":"setWord"}

async fn philologus_defs(
    (info, req): (web::Query<DefRequest>, HttpRequest),
) -> Result<HttpResponse, AWError> {
    let db = req.app_data::<SqlitePool>().unwrap();
    let db2 = req.app_data::<SqliteUpdatePool>().unwrap();

    let table = get_long_lex(info.lexicon.as_str());

    let def_row = if info.wordid.is_some() {
        let decoded_word = percent_decode_str(info.wordid.as_ref().unwrap())
            .decode_utf8()
            .map_err(map_utf8_error)?;
        get_def_by_word(db, table, &decoded_word)
            .await
            .map_err(map_sqlx_error)?
    } else if info.id.is_some() {
        get_def_by_seq(db, info.id.unwrap())
            .await
            .map_err(map_sqlx_error)?
    } else {
        return Err(PhilologusError {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            name: "philologus_defs error: word and id are both empty".to_string(),
            error: "philologus_defs error: word and id are both empty".to_string(),
        }
        .into());
    };

    let lex = match info.lexicon.as_str() {
        "ls" => 1,
        "slater" => 2,
        _ => 0,
    };
    let time_stamp = SystemTime::now().duration_since(UNIX_EPOCH);
    let time_stamp_ms = if time_stamp.is_ok() {
        time_stamp.unwrap().as_millis()
    } else {
        0
    };
    let user_agent = get_user_agent(&req).unwrap_or("");
    //https://stackoverflow.com/questions/66989780/how-to-retrieve-the-ip-address-of-the-client-from-httprequest-in-actix-web
    let ip = if req.peer_addr().is_some() {
        req.peer_addr().unwrap().ip().to_string()
    } else {
        "".to_string()
    };
    let _ = insert_log(
        &db2.0,
        time_stamp_ms,
        lex,
        def_row.seq,
        ip.as_str(),
        user_agent,
    )
    .await;

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
        method: "setWord".to_string(),
    };

    Ok(HttpResponse::Ok().json(res))
}

fn add_bibl_links(def: &str) -> String {
    let re = Regex::new(r#"biblink="(?P<l>[^"]*)""#).unwrap();
    let def_with_links = re.replace_all(def, r#"href="http://www.perseus.tufts.edu/hopper/text.jsp?doc=$l&amp;lang=original" target="_blank""#);

    def_with_links.to_string()
}

// this is for a route to specific word: /{lex}/{word}
async fn index(_req: HttpRequest) -> Result<NamedFile> {
    let path: PathBuf = "static/index.html".parse().unwrap();
    Ok(NamedFile::open(path)?)
}

async fn health_check(_req: HttpRequest) -> Result<HttpResponse, AWError> {
    Ok(HttpResponse::Ok().finish()) //send 200 with empty body
}

async fn synopsis_list(req: HttpRequest) -> Result<HttpResponse, AWError> {
    let db2 = req.app_data::<SqliteUpdatePool>().unwrap();

    let list = get_synopsis_list(&db2.0).await.map_err(map_sqlx_error)?;

    let mut res = String::from(
        r#"<!DOCTYPE html>
    <html>
    <head>
    <meta charset="UTF-8">
    </head>
    <body><table>"#,
    );
    for l in list {
        let d = UNIX_EPOCH + Duration::from_millis(l.1.try_into().unwrap());
        let datetime = DateTime::<Local>::from(d);
        let timestamp_str = datetime.format("%Y-%m-%d %H:%M:%S").to_string();

        res.push_str(format!("<tr><td><a href='synopsisresult?sid={}'>{}</a></td><td>{}</td><td>{}</td><td>{}</td></tr>", l.0, timestamp_str, l.2, l.3,l.4).as_str());
    }
    res.push_str("</table></body></html>");

    Ok(HttpResponse::Ok().content_type("text/html").body(res))
}

async fn synopsis_result(
    (info, req): (web::Query<SynopsisResultRequest>, HttpRequest),
) -> Result<HttpResponse, AWError> {
    let db2 = req.app_data::<SqliteUpdatePool>().unwrap();

    let list = get_synopsis_result(&db2.0, info.id)
        .await
        .map_err(map_sqlx_error)?;

    let mut res = String::from(
        r#"<!DOCTYPE html>
    <html>
    <head>
    <meta charset="UTF-8">
    </head>
    <body><table>"#,
    );
    for l in list {
        let d = UNIX_EPOCH + Duration::from_millis(l.1.try_into().unwrap());
        let datetime = DateTime::<Local>::from(d);
        let timestamp_str = datetime.format("%Y-%m-%d %H:%M:%S").to_string();

        res.push_str(format!("<tr><td><a href='synopsisresult?sid={}'>{}</a></td><td>{}</td><td>{}</td><td>{}</td></tr>", l.0, timestamp_str, l.2, l.3,l.4).as_str());
    }
    res.push_str("</table></body></html>");

    Ok(HttpResponse::Ok().content_type("text/html").body(res))
}

async fn synopsis_saver(
    (info, req): (web::Json<SynopsisSaverRequest>, HttpRequest),
) -> Result<HttpResponse, AWError> {
    let db2 = req.app_data::<SqliteUpdatePool>().unwrap();

    let time_stamp = SystemTime::now().duration_since(UNIX_EPOCH);
    let time_stamp_ms = if time_stamp.is_ok() {
        time_stamp.unwrap().as_millis()
    } else {
        0
    };
    let user_agent = get_user_agent(&req).unwrap_or("");
    //https://stackoverflow.com/questions/66989780/how-to-retrieve-the-ip-address-of-the-client-from-httprequest-in-actix-web
    let ip = if req.peer_addr().is_some() {
        req.peer_addr().unwrap().ip().to_string()
    } else {
        "".to_string()
    };

    let _ = insert_synopsis(
        &db2.0,
        &info.into_inner(),
        time_stamp_ms,
        ip.as_str(),
        user_agent,
    )
    .await
    .map_err(map_sqlx_error)?;

    //Ok(HttpResponse::Ok().finish())
    //let res = 1;
    Ok(HttpResponse::Ok().json(1))
}

async fn synopsis(_req: HttpRequest) -> Result<HttpResponse, AWError> {
    let mut template = include_str!("synopsis.html").to_string();

    let mut rows = String::from("");
    let mut count = 0;
    let rowlabels = vec![
        "Present Indicative",
        "Future Indicative",
        "Imperfect Indicative",
        "Aorist Indicative",
        "Perfect Indicative",
        "Pluperfect Indicative",
        "Present Subjunctive",
        "Aorist Subjunctive",
        "Present Optative",
        "Future Optative",
        "Aorist Optative",
        "Present Imperative",
        "Aorist Imperative",
        "Present Infinitive",
        "Future Infinitive",
        "Aorist Infinitive",
        "Perfect Infinitive",
        "Present Participle",
        "Future Participle",
        "Aorist Participle",
        "Perfect Participle",
    ];
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

    Ok(HttpResponse::Ok().content_type("text/html").body(template))
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

use actix_web::dev::Service;
use actix_web::http::header::{HeaderValue, CONTENT_SECURITY_POLICY};

#[actix_web::main]
async fn main() -> io::Result<()> {
    std::env::set_var("RUST_LOG", "trace");
    // env_logger::init();

    //e.g. export PHILOLOGUS_DB_PATH=sqlite://philolog_us_local.sqlite?mode=ro
    //e.g. export PHILOLOGUS_LOG_DB_PATH=sqlite://log.sqlite?mode=rwc
    //e.g. export TANTIVY_INDEX_PATH=/Users/jeremy/Documents/code/tantivy-test/tantivy-data
    //e.g. export TRACING_LOG_PATH=/Users/jeremy/Documents/code/phlogs
    let db_path = std::env::var("PHILOLOGUS_DB_PATH").unwrap_or_else(|_| {
        panic!("Environment variable for sqlite path not set: PHILOLOGUS_DB_PATH.")
    });
    let db_log_path = std::env::var("PHILOLOGUS_LOG_DB_PATH").unwrap_or_else(|_| {
        panic!("Environment variable for sqlite log path not set: PHILOLOGUS_LOG_DB_PATH.")
    });
    let tantivy_index_path = std::env::var("TANTIVY_INDEX_PATH").unwrap_or_else(|_| {
        panic!("Environment variable for tantivy index path not set: TANTIVY_INDEX_PATH.")
    });
    let tracing_log_path = std::env::var("TRACING_LOG_PATH").unwrap_or_else(|_| {
        panic!("Environment variable for tracing log path not set: TRACING_LOG_PATH.")
    });

    // Log all events to a rolling log file.
    let logfile = tracing_appender::rolling::never(tracing_log_path, "philo-logs");
    let (non_blocking, _guard) = tracing_appender::non_blocking(logfile);
    // Log `INFO` and above to stdout.
    let stdout = std::io::stdout.with_max_level(tracing::Level::INFO);
    tracing_subscriber::fmt()
        // Combine the stdout and log file `MakeWriter`s into one
        // `MakeWriter` that writes to both
        .with_writer(stdout.and(non_blocking))
        .init();

    let db_pool = SqlitePool::connect(&db_path)
        .await
        .expect("Could not connect to db.");

    let db_log_pool = SqliteUpdatePool(
        SqlitePool::connect(&db_log_path)
            .await
            .expect("Could not connect to update db."),
    );
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

    let tantivy_index = Index::open_in_dir(tantivy_index_path).unwrap();

    HttpServer::new(move || {
        App::new()
            .app_data(tantivy_index.clone())
            .app_data(db_pool.clone())
            .app_data(db_log_pool.clone())
            //.wrap(middleware::Logger::default())
            .wrap(TracingLogger::default())
            .wrap(middleware::Compress::default())
            // .app_data(
            //     web::JsonConfig::default()
            //         .error_handler(json_error_handler)
            //         .limit(262_144),
            // )
            .app_data(web::PayloadConfig::default().limit(262_144))
            .wrap_fn(|req, srv| {
                let fut = srv.call(req);
                async {
                    let mut res = fut.await?;
                    res.headers_mut()
                        .insert(CONTENT_SECURITY_POLICY, HeaderValue::from_static("script-src 'nonce-2726c7f26c' 'unsafe-inline'; object-src 'none'; base-uri 'none'"));
                    Ok(res)
                }
            })
            //.wrap(error_handlers)
            .service(web::resource("/{lex}/query").route(web::get().to(philologus_words)))
            .service(web::resource("/{lex}/item").route(web::get().to(philologus_defs)))
            .service(web::resource("/{lex}/ft/").route(web::get().to(full_text_query)))
            .service(web::resource("/{lex}/{word}").route(web::get().to(index))) //requesting page from a word link, order of services matters
            .service(web::resource("/ft").route(web::get().to(full_text_query)))
            .service(web::resource("/item").route(web::get().to(philologus_defs)))
            .service(web::resource("/query").route(web::get().to(philologus_words)))
            .service(web::resource("/healthzzz").route(web::get().to(health_check)))
            .service(web::resource("/synopsisresult").route(web::get().to(synopsis_result)))
            .service(web::resource("/synopsislist").route(web::get().to(synopsis_list)))
            .service(web::resource("/synopsissaver").route(web::post().to(synopsis_saver)))
            .service(web::resource("/synopsis").route(web::get().to(synopsis)))
            .service(web::resource("/hc.php").route(web::get().to(hc)))
            .service(
                fs::Files::new("/", "./static")
                    .prefer_utf8(true)
                    .index_file("index.html"),
            )
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
        write!(
            fmt,
            "PhilologusError: {} {}: {}.",
            self.code.as_u16(),
            self.name,
            self.error
        )
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
        sqlx::Error::Configuration(e) => PhilologusError {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            name: "sqlx error".to_string(),
            error: format!("sqlx Configuration: {}", e),
        },
        sqlx::Error::Database(e) => PhilologusError {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            name: "sqlx error".to_string(),
            error: format!("sqlx Database: {}", e),
        },
        sqlx::Error::Io(e) => PhilologusError {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            name: "sqlx error".to_string(),
            error: format!("sqlx Io: {}", e),
        },
        sqlx::Error::Tls(e) => PhilologusError {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            name: "sqlx error".to_string(),
            error: format!("sqlx Tls: {}", e),
        },
        sqlx::Error::Protocol(e) => PhilologusError {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            name: "sqlx error".to_string(),
            error: format!("sqlx Protocol: {}", e),
        },
        sqlx::Error::RowNotFound => PhilologusError {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            name: "sqlx error".to_string(),
            error: "sqlx RowNotFound".to_string(),
        },
        sqlx::Error::TypeNotFound { .. } => PhilologusError {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            name: "sqlx error".to_string(),
            error: "sqlx TypeNotFound".to_string(),
        },
        sqlx::Error::ColumnIndexOutOfBounds { .. } => PhilologusError {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            name: "sqlx error".to_string(),
            error: "sqlx ColumnIndexOutOfBounds".to_string(),
        },
        sqlx::Error::ColumnNotFound(e) => PhilologusError {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            name: "sqlx error".to_string(),
            error: format!("sqlx ColumnNotFound: {}", e),
        },
        sqlx::Error::ColumnDecode { .. } => PhilologusError {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            name: "sqlx error".to_string(),
            error: "sqlx ColumnDecode".to_string(),
        },
        sqlx::Error::Decode(e) => PhilologusError {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            name: "sqlx error".to_string(),
            error: format!("sqlx Decode: {}", e),
        },
        sqlx::Error::PoolTimedOut => PhilologusError {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            name: "sqlx error".to_string(),
            error: "sqlx PoolTimeOut".to_string(),
        },
        sqlx::Error::PoolClosed => PhilologusError {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            name: "sqlx error".to_string(),
            error: "sqlx PoolClosed".to_string(),
        },
        sqlx::Error::WorkerCrashed => PhilologusError {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            name: "sqlx error".to_string(),
            error: "sqlx WorkerCrashed".to_string(),
        },
        sqlx::Error::Migrate(e) => PhilologusError {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            name: "sqlx error".to_string(),
            error: format!("sqlx Migrate: {}", e),
        },
        _ => PhilologusError {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            name: "sqlx error".to_string(),
            error: "sqlx Unknown error".to_string(),
        },
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    code: u16,
    error: String,
    message: String,
}

fn map_utf8_error(_e: std::str::Utf8Error) -> PhilologusError {
    PhilologusError {
        code: StatusCode::INTERNAL_SERVER_ERROR,
        name: "percent_decode_str utf-8 error".to_string(),
        error: "percent_decode_str utf-8 error".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App};
    use urlencoding::encode;

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
        let s = r#"{"pp":"ἵστημι, στήσω, ἔστησα / ἔστην, ἕστηκα, ἕσταμαι, ἐστάθην","unit":"22","verb":"αα","person":"2nd","number":"sing","ptccase":"dat","ptcgender":"fem","ptcnumber":"","sname":"name","advisor":"advisor","r":["ββ","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","γγ"]}"#;
        let r: SynopsisSaverRequest = serde_json::from_str(s).unwrap();
        assert_eq!(r.verb, "αα".to_string());
        assert_eq!(r.r[62], "γγ".to_string());
    }

    #[test]
    async fn test_unicode_strip_diacritics_and_lowercase() {
        let a = "ἄέώΏ"
            .nfd()
            .filter(|x| !unicode_normalization::char::is_combining_mark(*x))
            .collect::<String>()
            .to_lowercase();
        assert_eq!(a, "αεωω");
    }

    #[test]
    async fn test_add_links() {
        let a = r#"blah biblink="Perseus:abo:tlg,0059,005:405c"> blahblah biblink="Perseus:abo:tlg,4083,001:641:61">"#;
        let b = add_bibl_links(a);
        assert_eq!(
            b,
            r#"blah href="http://www.perseus.tufts.edu/hopper/text.jsp?doc=Perseus:abo:tlg,0059,005:405c&amp;lang=original" target="_blank"> blahblah href="http://www.perseus.tufts.edu/hopper/text.jsp?doc=Perseus:abo:tlg,4083,001:641:61&amp;lang=original" target="_blank">"#
        );
    }

    #[actix_web::test]
    async fn test_query_paging_short() {
        let db_path = "sqlite::memory:";

        let db_pool = SqlitePool::connect(db_path)
            .await
            .expect("Could not connect to db.");

        let db_pool2 = SqliteUpdatePool(
            SqlitePool::connect(db_path)
                .await
                .expect("Could not connect to update db."),
        );

        let app = test::init_service(
            App::new()
                .app_data(db_pool.clone())
                .app_data(db_pool2.clone())
                .service(web::resource("/{lex}/query").route(web::get().to(philologus_words)))
                .service(web::resource("/healthzzz").route(web::get().to(health_check)))
                .service(web::resource("/hc.php").route(web::get().to(hc)))
                .service(web::resource("/{lex}/item").route(web::get().to(philologus_defs))),
        )
        .await;

        let query = "CREATE TABLE words (seq INTEGER PRIMARY KEY, lexicon TEXT, word TEXT, sortword TEXT, def TEXT);";
        sqlx::query(query).execute(&db_pool).await.unwrap();

        let query = "INSERT INTO words VALUES (NULL, $1, $2, $3, $4);";
        for i in &["α", "β", "γ", "δ", "ε"] {
            sqlx::query(query)
                .bind("lsj")
                .bind(i)
                .bind(i)
                .bind("def")
                .execute(&db_pool)
                .await
                .unwrap();
        }

        //empty query
        let query = r#"{"regex":0,"lexicon":"lsj","tag_id":0,"root_id":0,"w":""}"#;
        let encoded_url = format!(
            "/lsj/query?n={}&idprefix={}\
            &x=0.795795025371805&requestTime=1637859894040&\
            page={}&mode=context&query={}",
            5,
            "test1",
            0,
            encode(query)
        );

        let resp = test::TestRequest::get()
            .uri(&encoded_url)
            .send_request(&app)
            .await;

        assert!(&resp.status().is_success());
        let result: QueryResponse = test::read_body_json(resp).await;
        assert_eq!(result.arr_options.len(), 5);

        assert_eq!(result.arr_options[0].1, 1);
        assert_eq!(result.arr_options[result.arr_options.len() - 1].1, 5);
        assert_eq!(result.lastpage_up, 1);
        assert_eq!(result.last_page, 0);
        assert_eq!(result.page, 0);
        assert_eq!(result.select_id, 0);

        //query α
        let query = r#"{"regex":0,"lexicon":"lsj","tag_id":0,"root_id":0,"w":"α"}"#;
        let encoded_url = format!(
            "/lsj/query?n={}&idprefix={}\
            &x=0.795795025371805&requestTime=1637859894040&\
            page={}&mode=context&query={}",
            5,
            "test1",
            0,
            encode(query)
        );

        let resp = test::TestRequest::get()
            .uri(&encoded_url)
            .send_request(&app)
            .await;

        assert!(&resp.status().is_success());
        let result: QueryResponse = test::read_body_json(resp).await;
        assert_eq!(result.arr_options.len(), 5);

        assert_eq!(result.arr_options[0].1, 1);
        assert_eq!(result.arr_options[result.arr_options.len() - 1].1, 5);
        assert_eq!(result.lastpage_up, 1);
        assert_eq!(result.last_page, 0);
        assert_eq!(result.page, 0);
        assert_eq!(result.select_id, 1);

        //query γ
        let query = r#"{"regex":0,"lexicon":"lsj","tag_id":0,"root_id":0,"w":"γ"}"#;
        let encoded_url = format!(
            "/lsj/query?n={}&idprefix={}\
            &x=0.795795025371805&requestTime=1637859894040&\
            page={}&mode=context&query={}",
            5,
            "test1",
            0,
            encode(query)
        );

        let resp = test::TestRequest::get()
            .uri(&encoded_url)
            .send_request(&app)
            .await;

        assert!(&resp.status().is_success());
        let result: QueryResponse = test::read_body_json(resp).await;
        assert_eq!(result.arr_options.len(), 5);

        assert_eq!(result.arr_options[0].1, 1);
        assert_eq!(result.arr_options[result.arr_options.len() - 1].1, 5);
        assert_eq!(result.lastpage_up, 1);
        assert_eq!(result.last_page, 1);
        assert_eq!(result.page, 0);
        assert_eq!(result.select_id, 3);

        //query ω
        let query = r#"{"regex":0,"lexicon":"lsj","tag_id":0,"root_id":0,"w":"ω"}"#;
        let encoded_url = format!(
            "/lsj/query?n={}&idprefix={}\
            &x=0.795795025371805&requestTime=1637859894040&\
            page={}&mode=context&query={}",
            5,
            "test1",
            0,
            encode(query)
        );

        let resp = test::TestRequest::get()
            .uri(&encoded_url)
            .send_request(&app)
            .await;

        assert!(&resp.status().is_success());
        let result: QueryResponse = test::read_body_json(resp).await;
        assert_eq!(result.arr_options.len(), 5);

        assert_eq!(result.arr_options[0].1, 1);
        assert_eq!(result.arr_options[result.arr_options.len() - 1].1, 5);
        assert_eq!(result.lastpage_up, 1);
        assert_eq!(result.last_page, 1);
        assert_eq!(result.page, 0);
        assert_eq!(result.select_id, 5);

        //DefResponse
        let resp = test::TestRequest::get()
            .uri(r#"/lsj/item?id=1&lexicon=lsj&skipcache=0&addwordlinks=0&x=0.7049151126608002"#)
            .send_request(&app)
            .await;
        let result: DefResponse = test::read_body_json(resp).await;

        assert_eq!(result.word_id, 1);

        //DefResponse: both word and id are empty
        let resp = test::TestRequest::get()
            .uri(r#"/lsj/item?lexicon=lsj&skipcache=0&addwordlinks=0&x=0.7049151126608002"#)
            .send_request(&app)
            .await;
        assert_eq!(resp.status(), 500);
    }
}
