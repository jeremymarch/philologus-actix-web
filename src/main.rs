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

use actix_files as fs;
use actix_web::{middleware, web, App, Error as AWError, HttpResponse, HttpServer};
use r2d2_sqlite::{self, SqliteConnectionManager};

mod db;
use db::{Pool,Info,WordQuery,GreekWords};
use serde::{Deserialize, Serialize};

/*
{"error":"","wtprefix":"test1","nocache":"1","container":"test1Container","requestTime":"1635643672625","selectId":"32","page":"0","lastPage":"0","lastPageUp":"1","scroll":"32","query":"","arrOptions":[{"i":1,"r":["Α α",1,0]},{"i":2,"r":["ἀ-",2,0]},{"i":3,"r":["ἀ-",3,0]},{"i":4,"r":["ἆ",4,0]}...
*/

#[derive(Debug, Serialize, Deserialize, Clone)]
struct output {
    error: String,
    wtprefix: String,
    nocache: String,
    container: String,
    requestTime: String,
    selectId: String,
    page: String,
    lastPage: String,
    lastpageUp: String,
    scroll: String,
    query: String,
    arrOptions: Vec<GreekWords>
}

//http://127.0.0.1:8080/philwords?n=101&idprefix=test1&x=0.1627681205837177&requestTime=1635643672625&page=0&mode=context&query={%22regex%22:%220%22,%22lexicon%22:%22lsj%22,%22tag_id%22:%220%22,%22root_id%22:%220%22,%22wordid%22:%22%CE%B1%CE%B1%CF%84%CE%BF%CF%832%22,%22w%22:%22%22}

#[allow(clippy::eval_order_dependence)]
async fn philologus_words((db, info): (web::Data<Pool>, web::Query<Info>)) -> Result<HttpResponse, AWError> {
    let p: WordQuery = serde_json::from_str(&info.query)?;
    
    let seq = db::execute_get_seq(&db,&p).await?;
    let mut result = db::execute(&db, seq, true, &p).await?;
    result.reverse();
    let result2 = db::execute(&db, seq, false, &p).await?;
    let result = [result, result2].concat();

    let res = output {
        error: "".to_owned(),
        wtprefix: info.idprefix.clone(),
        nocache: "1".to_owned(),
        container: format!("{}Container", info.idprefix),
        requestTime: info.requestTime.to_string(),
        selectId: seq.to_string(),
        page: "0".to_owned(),
        lastPage: "1".to_owned(),
        lastpageUp: "1".to_owned(),
        scroll: seq.to_string(),
        query: "".to_owned(),
        arrOptions: result
    };

    Ok(HttpResponse::Ok().json(res))
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let manager = SqliteConnectionManager::file("philolog_us.sqlite");
    let pool = Pool::new(manager).unwrap();

    HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .wrap(middleware::Logger::default())
            .service(
                web::resource("/wtgreekserv.php")
                    .route(web::get().to(philologus_words)),
            )
            .service(fs::Files::new("/", "static").prefer_utf8(true).index_file("index.html"))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
