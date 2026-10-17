/*
philologus-axum: philologus is a collection of digitized Greek and Latin Lexica

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

use axum::extract::{ConnectInfo, Query, State};
use axum::http::header::{CONTENT_SECURITY_POLICY, CONTENT_TYPE, STRICT_TRANSPORT_SECURITY};
use axum::http::request::Parts;
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, get_service, post};
use axum::{Json, Router};
use tower_http::compression::CompressionLayer;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};

use chrono::prelude::*;
use percent_encoding::percent_decode_str;
use regex::Regex;
use sqlx::SqlitePool;
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use thiserror::Error;
use unicode_normalization::UnicodeNormalization;

use libphilologus::*;
mod synopsis;
use crate::synopsis::*;
use serde::{Deserialize, Serialize};

use std::time::{SystemTime, UNIX_EPOCH};

use tantivy::collector::{Count, TopDocs};
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::{Index, ReloadPolicy};

use hoplite_verbs_rs::{HcGreekVerb, hgk_strip_diacritics};

// static files are shared with the actix crate; path is relative to the working directory
const STATIC_DIR: &str = "philologus-actix/static";

#[derive(Clone)]
struct SqliteUpdatePool(SqlitePool);

#[derive(Clone)]
struct AppState {
    db: SqlitePool,
    // only set if env PHILOLOGUS_LOG_DB_PATH is set
    update_db: Option<SqliteUpdatePool>,
    index: Index,
    verbs: Arc<Vec<Arc<HcGreekVerb>>>,
}

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
    arr_options: Vec<(u32, String)>,
}

struct QueryResult {
    seq: u32,
    vlast_page_up: u8,
    vlast_page: u8,
    rows: Vec<(u32, String)>,
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
    pub p: usize,
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
    page: usize,
    limit: usize,
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
pub struct SynopsisResultRequest {
    pub id: u32,
}

async fn full_text_query(
    State(state): State<AppState>,
    Query(info): Query<FullTextQueryRequest>,
) -> Result<Response, PhilologusError> {
    let db = &state.db;
    let index = &state.index;

    let limit = 20;
    let offset = info.p * 20;

    let lexicon_field = index.schema().get_field("lexicon").unwrap();
    let definition_field = index.schema().get_field("definition").unwrap();

    let reader = index
        .reader_builder()
        .reload_policy(ReloadPolicy::OnCommitWithDelay)
        .try_into()
        .unwrap();

    let searcher = reader.searcher();
    let mut query_parser = QueryParser::for_index(
        index,
        //this vector contains default fields used if field is not specified in query
        vec![lexicon_field, definition_field],
    );
    query_parser.set_conjunction_by_default(); // AND by default

    let mut res = FullTextResponse {
        ftresults: vec![],
        error: String::from(""),
        count: 0,
        page: info.p,
        limit,
    };

    // full-text index should be all lowercase, but use uppercase for AND and OR
    let mut ft_query = info.q.to_lowercase();
    ft_query = hgk_strip_diacritics(
        ft_query
            .replace(" and ", " AND ")
            .replace(" or ", " OR ")
            .as_str(),
        0xFFFFFFFF,
    );

    let my_collector = (
        Count,
        TopDocs::with_limit(limit)
            .and_offset(offset)
            .order_by_score(),
    );
    match query_parser.parse_query(&ft_query) {
        Ok(query) => match searcher.search(&query, &my_collector) {
            Ok((count, top_docs)) => {
                for (_score, doc_address) in top_docs {
                    match searcher.doc::<TantivyDocument>(doc_address) {
                        Ok(retrieved_doc) => {
                            let mut word_id_value: u32 = 0;
                            let mut lexicon_value: String = String::from("");

                            for (field, field_values) in retrieved_doc.get_sorted_field_values() {
                                match index.schema().get_field_name(field) {
                                    "lexicon" => {
                                        lexicon_value =
                                            field_values[0].as_str().unwrap_or("").to_string()
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
    Ok(Json(res).into_response())
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
) -> Result<QueryResult, PhilologusError> {
    let seq = if let Some(wid) = &word_id {
        let decoded_word = percent_decode_str(wid)
            .decode_utf8()
            .map_err(map_utf8_error)?;
        get_seq_by_word(db, table, &decoded_word).await.unwrap()
    } else {
        let query = sanitize_query(query);
        get_seq_by_prefix(db, table, &query).await.unwrap()
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
            row.1 = re.replace_all(&row.1, "").to_string();
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
    State(state): State<AppState>,
    Query(info): Query<QueryRequest>,
) -> Result<Response, PhilologusError> {
    let query_params: WordQuery = serde_json::from_str(&info.query)?;

    let table = get_long_lex(query_params.lexicon.as_str());

    let query_result = query_words(
        &state.db,
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

    Ok(Json(res).into_response())
}

fn get_user_agent(headers: &HeaderMap) -> Option<&str> {
    headers.get("user-agent")?.to_str().ok()
}

fn get_ip(parts: &Parts) -> String {
    parts
        .extensions
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ci| ci.0.ip().to_string())
        .unwrap_or_default()
}

fn get_long_lex(lex: &str) -> &str {
    match lex {
        "ls" => "lewisshort",
        "slater" => "slater",
        _ => "lsj",
    }
}

async fn philologus_defs(
    State(state): State<AppState>,
    parts: Parts,
    Query(info): Query<DefRequest>,
) -> Result<Response, PhilologusError> {
    let db = &state.db;

    let table = get_long_lex(info.lexicon.as_str());

    let def_row = if let Some(wid) = &info.wordid {
        let decoded_word = percent_decode_str(wid)
            .decode_utf8()
            .map_err(map_utf8_error)?;
        get_def_by_word(db, table, &decoded_word)
            .await
            .map_err(map_sqlx_error)?
    } else if let Some(id) = info.id {
        get_def_by_seq(db, id).await.map_err(map_sqlx_error)?
    } else {
        return Err(PhilologusError {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            name: "philologus_defs error: word and id are both empty".to_string(),
            error: "philologus_defs error: word and id are both empty".to_string(),
        });
    };

    let lex = match info.lexicon.as_str() {
        "ls" => 1,
        "slater" => 2,
        _ => 0,
    };
    let time_stamp = SystemTime::now().duration_since(UNIX_EPOCH);
    let time_stamp_ms = if let Ok(time_stamp) = time_stamp {
        time_stamp.as_millis()
    } else {
        0
    };
    let user_agent = get_user_agent(&parts.headers).unwrap_or("");
    let ip = get_ip(&parts);

    // only log request to db if env PHILOLOGUS_LOG_DB_PATH is set
    if let Some(update_db) = &state.update_db {
        let _ = insert_log(
            &update_db.0,
            time_stamp_ms,
            lex,
            def_row.seq,
            ip.as_str(),
            user_agent,
        )
        .await;
    }

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

    Ok(Json(res).into_response())
}

fn add_bibl_links(def: &str) -> String {
    let re = Regex::new(r#"biblink="(?P<l>[^"]*)""#).unwrap();
    let def_with_links = re.replace_all(def, r#"href="http://www.perseus.tufts.edu/hopper/text.jsp?doc=$l&amp;lang=original" target="_blank""#);

    def_with_links.to_string()
}

async fn health_check() -> StatusCode {
    StatusCode::OK //send 200 with empty body
}

async fn hc() -> StatusCode {
    StatusCode::OK
}

// match actix-files' prefer_utf8: static text files are served with charset=utf-8
async fn add_utf8_charset(mut res: Response) -> Response {
    if let Some(ct) = res
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        && (ct.starts_with("text/") || ct == "application/javascript")
        && !ct.contains("charset")
        && let Ok(v) = HeaderValue::from_str(&format!("{}; charset=utf-8", ct))
    {
        res.headers_mut().insert(CONTENT_TYPE, v);
    }
    res
}

fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/{lex}/query", get(philologus_words))
        .route("/{lex}/item", get(philologus_defs))
        .route("/{lex}/ft/", get(full_text_query))
        // requesting page from a word link
        .route(
            "/{lex}/{word}",
            get_service(ServeFile::new(format!("{}/index.html", STATIC_DIR))),
        )
        .route("/ft/", get(full_text_query))
        .route("/item", get(philologus_defs))
        .route("/query", get(philologus_words))
        .route("/healthzzz", get(health_check))
        .route("/greek-synopsis-result", get(greek_synopsis_result))
        .route("/greek-synopsis-list", get(greek_synopsis_list))
        .route("/greek-synopsis-saver", post(greek_synopsis_saver))
        .route("/latin-synopsis-result", get(latin_synopsis_result))
        .route("/latin-synopsis-list", get(latin_synopsis_list))
        .route("/latin-synopsis-saver", post(latin_synopsis_saver))
        .route("/latin-synopsis", get(latin_synopsis))
        .route("/synopsis-json", post(synopsis_json))
        .route("/cetest", get(cetest))
        .route("/hc.php", get(hc))
        .fallback_service(ServeDir::new(STATIC_DIR))
        .with_state(state)
        // layers added later wrap the ones added earlier
        .layer(axum::middleware::map_response(add_utf8_charset))
        .layer(SetResponseHeaderLayer::if_not_present(
            CONTENT_SECURITY_POLICY,
            HeaderValue::from_static("style-src 'nonce-2726c7f26c';\
                script-src 'nonce-2726c7f26c' 'wasm-unsafe-eval' 'unsafe-inline'; object-src 'none'; base-uri 'none'"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            STRICT_TRANSPORT_SECURITY,
            HeaderValue::from_static("max-age=31536000" /* 1 year */),
        ))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(tracing::Level::INFO))
                .on_response(DefaultOnResponse::new().level(tracing::Level::INFO)),
        )
        .layer(CompressionLayer::new())
}

#[tokio::main]
async fn main() -> io::Result<()> {
    //e.g. export PHILOLOGUS_DB_PATH=sqlite://db.sqlite?mode=ro
    //e.g. export PHILOLOGUS_LOG_DB_PATH=sqlite://log.sqlite?mode=rwc
    //e.g. export TANTIVY_INDEX_PATH=/Users/jeremy/Documents/code/philologus-lex-loader/tantivy-data
    let db_path = std::env::var("PHILOLOGUS_DB_PATH").unwrap_or_else(|_| {
        panic!("Environment variable for sqlite path not set: PHILOLOGUS_DB_PATH.")
    });
    let tantivy_index_path = std::env::var("TANTIVY_INDEX_PATH").unwrap_or_else(|_| {
        panic!("Environment variable for tantivy index path not set: TANTIVY_INDEX_PATH.")
    });

    // Log `INFO` and above to stdout.
    let stdout = std::io::stdout.with_max_level(tracing::Level::INFO);
    tracing_subscriber::fmt().with_writer(stdout).init();

    let db_pool = SqlitePool::connect(&db_path)
        .await
        .expect("Could not connect to db.");

    // if env PHILOLOGUS_LOG_DB_PATH is set, set up log db
    let db_log_pool;
    if let Ok(db_log_path) = std::env::var("PHILOLOGUS_LOG_DB_PATH") {
        db_log_pool = Some(SqliteUpdatePool(
            SqlitePool::connect(&db_log_path)
                .await
                .expect("Could not connect to update db."),
        ));
        if let Some(ref update_db) = db_log_pool {
            let query = "CREATE TABLE IF NOT EXISTS log (id integer primary key autoincrement, accessed integer, lexicon integer, word integer, ip text, agent text);";
            let _ = sqlx::query(query).execute(&update_db.0).await;
        }
    } else {
        db_log_pool = None;
    }

    let tantivy_index = Index::open_in_dir(tantivy_index_path).unwrap();

    let state = AppState {
        db: db_pool,
        update_db: db_log_pool,
        index: tantivy_index,
        verbs: Arc::new(load_verbs("pp.txt")),
    };

    let app = create_router(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8088").await?;
    tracing::info!("listening on {}", listener.local_addr()?);
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
}

// actix-web shuts down on these signals by default; axum must be told to.
// this matters in docker, where the server is PID 1 and gets no default signal handling
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    tracing::info!("shutting down");
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

impl IntoResponse for PhilologusError {
    fn into_response(self) -> Response {
        let error_response = ErrorResponse {
            code: self.code.as_u16(),
            message: self.error,
            error: self.name,
        };
        (self.code, Json(error_response)).into_response()
    }
}

impl From<serde_json::Error> for PhilologusError {
    fn from(e: serde_json::Error) -> PhilologusError {
        PhilologusError {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            name: "serde_json error".to_string(),
            error: e.to_string(),
        }
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
    use axum::body::Body;
    use axum::http::Request;
    use http_body_util::BodyExt;
    use tower::ServiceExt;
    use urlencoding::encode;

    async fn get_request(app: &Router, uri: &str) -> Response {
        app.clone()
            .oneshot(Request::get(uri).body(Body::empty()).unwrap())
            .await
            .unwrap()
    }

    async fn read_body_json<T: serde::de::DeserializeOwned>(resp: Response) -> T {
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[test]
    fn test_unicode_strip_diacritics_and_lowercase() {
        let a = "ἄέώΏ"
            .nfd()
            .filter(|x| !unicode_normalization::char::is_combining_mark(*x))
            .collect::<String>()
            .to_lowercase();
        assert_eq!(a, "αεωω");
    }

    #[test]
    fn test_add_links() {
        let a = r#"blah biblink="Perseus:abo:tlg,0059,005:405c"> blahblah biblink="Perseus:abo:tlg,4083,001:641:61">"#;
        let b = add_bibl_links(a);
        assert_eq!(
            b,
            r#"blah href="http://www.perseus.tufts.edu/hopper/text.jsp?doc=Perseus:abo:tlg,0059,005:405c&amp;lang=original" target="_blank"> blahblah href="http://www.perseus.tufts.edu/hopper/text.jsp?doc=Perseus:abo:tlg,4083,001:641:61&amp;lang=original" target="_blank">"#
        );
    }

    #[tokio::test]
    async fn test_query_paging_short() {
        let db_path = "sqlite::memory:";

        let db_pool = SqlitePool::connect(db_path)
            .await
            .expect("Could not connect to db.");

        let db_pool2 = Some(SqliteUpdatePool(
            SqlitePool::connect(db_path)
                .await
                .expect("Could not connect to update db."),
        ));

        let app = create_router(AppState {
            db: db_pool.clone(),
            update_db: db_pool2,
            index: Index::create_in_ram(Schema::builder().build()),
            verbs: Arc::new(vec![]),
        });

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

        // (query word, expected lastPage, expected selectId)
        let cases = [("", 0, 0), ("α", 0, 1), ("γ", 1, 3), ("ω", 1, 5)];
        for (w, last_page, select_id) in cases {
            let query = format!(
                r#"{{"regex":0,"lexicon":"lsj","tag_id":0,"root_id":0,"w":"{}"}}"#,
                w
            );
            let encoded_url = format!(
                "/lsj/query?n={}&idprefix={}\
                &x=0.795795025371805&requestTime=1637859894040&\
                page={}&mode=context&query={}",
                5,
                "test1",
                0,
                encode(&query)
            );

            let resp = get_request(&app, &encoded_url).await;
            assert!(resp.status().is_success());
            let result: QueryResponse = read_body_json(resp).await;
            assert_eq!(result.arr_options.len(), 5);

            assert_eq!(result.arr_options[0].0, 1);
            assert_eq!(result.arr_options[result.arr_options.len() - 1].0, 5);
            assert_eq!(result.lastpage_up, 1);
            assert_eq!(result.last_page, last_page);
            assert_eq!(result.page, 0);
            assert_eq!(result.select_id, select_id);
        }

        //DefResponse
        let resp = get_request(
            &app,
            r#"/lsj/item?id=1&lexicon=lsj&skipcache=0&addwordlinks=0&x=0.7049151126608002"#,
        )
        .await;
        let result: DefResponse = read_body_json(resp).await;

        assert_eq!(result.word_id, 1);

        //DefResponse: both word and id are empty
        let resp = get_request(
            &app,
            r#"/lsj/item?lexicon=lsj&skipcache=0&addwordlinks=0&x=0.7049151126608002"#,
        )
        .await;
        assert_eq!(resp.status(), 500);
    }
}
