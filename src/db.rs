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

#[derive(Debug, Serialize, Deserialize)]
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
) -> impl Future<Output = Result<Vec<PhilologusWords>, AWError>> {
    let pool = pool.clone();
    web::block(move || {

        let result = get_words(pool.get()?);
        result.map_err(Error::from)
    })
    .map_err(AWError::from)
}

fn get_words(conn: Connection) -> PhilologusWordsResult {
    let stmt = conn.prepare(
        "
    SELECT seq,word
        FROM ZGREEK
        ORDER BY sortword ASC LIMIT 40",
    )?;
    get_words_res(stmt)
}

fn get_words_res(mut statement: Statement) -> PhilologusWordsResult {
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
{"error":"","wtprefix":"test1","nocache":"1","container":"test1Container","requestTime":"1635643672625","selectId":"32","page":"0","lastPage":"0","lastPageUp":"1","scroll":"32","query":"","arrOptions":[{"i":1,"r":["Α α",1,0]},{"i":2,"r":["ἀ-",2,0]},{"i":3,"r":["ἀ-",3,0]},{"i":4,"r":["ἆ",4,0]},{"i":5,"r":["ἃ ἅ",5,0]},{"i":6,"r":["ἄα",6,0]},{"i":7,"r":["ἀάατος",7,0]},{"i":8,"r":["ἀάβακτοι",8,0]},{"i":9,"r":["ἀαγής",9,0]},{"i":10,"r":["ἄαδα",10,0]},{"i":11,"r":["ἀαδένη",11,0]},{"i":12,"r":["ἀαδής",12,0]},{"i":13,"r":["ἀάζω",13,0]},{"i":14,"r":["ἄαθι",14,0]},{"i":15,"r":["ἀάκατος",15,0]},{"i":16,"r":["ἀακίδωτος",16,0]},{"i":17,"r":["ἀάλιον",17,0]},{"i":18,"r":["ἀανές",18,0]},{"i":19,"r":["ἄανθα",19,0]},{"i":20,"r":["ἀάπλετος",20,0]},{"i":21,"r":["ἄαπτος",21,0]},{"i":22,"r":["ἄας",22,0]},{"i":23,"r":["ἀασιφόρος",23,0]},{"i":24,"r":["ἀασιφρονία",24,0]},{"i":25,"r":["ἀασιφροσύνη",25,0]},{"i":26,"r":["ἀάσκει",26,0]},{"i":27,"r":["ἀασμός",27,0]},{"i":28,"r":["ἀάσπετος",28,0]},{"i":29,"r":["ἀάστονα",29,0]},{"i":30,"r":["ἀατήρ",30,0]},{"i":31,"r":["ἄατος",31,0]},{"i":32,"r":["ἄατος",32,0]},{"i":33,"r":["ἀάτυλον",33,0]},{"i":34,"r":["ἀάω",34,0]},{"i":35,"r":["ἄβα",35,0]},{"i":36,"r":["ἄβαγνα",36,0]},{"i":37,"r":["ἀβάδιστος",37,0]},{"i":38,"r":["ἀβαθής",38,0]},{"i":39,"r":["ἄβαθρος",39,0]},{"i":40,"r":["ἀβαίνω",40,0]},{"i":41,"r":["ἀβακέω",41,0]},{"i":42,"r":["ἀβακηνούς",42,0]},{"i":43,"r":["ἀβακής",43,0]},{"i":44,"r":["ἀβάκητον",44,0]},{"i":45,"r":["ἀβακίζομαι",45,0]},{"i":46,"r":["ἀβάκιον",46,0]},{"i":47,"r":["ἀβακίσκος",47,0]},{"i":48,"r":["ἀβακλή",48,0]},{"i":49,"r":["ἀβακοειδής",49,0]},{"i":50,"r":["ἄβακτον",50,0]},{"i":51,"r":["ἀβάκχευτος",51,0]},{"i":52,"r":["ἀβακχίωτος",52,0]},{"i":53,"r":["ἄβαλε",53,0]},{"i":54,"r":["ἀβαμβάκευτος",54,0]},{"i":55,"r":["ἄβαξ",55,0]},{"i":56,"r":["ἀβάπτιστος",56,0]},{"i":57,"r":["ἄβαπτος",57,0]},{"i":58,"r":["ἀβαρβάριστος",58,0]},{"i":59,"r":["ἀβαρής",59,0]},{"i":60,"r":["ἄβαρις",60,0]},{"i":61,"r":["ἀβασάνιστος",61,0]},{"i":62,"r":["ἀβασίλευτος",62,0]},{"i":63,"r":["ἀβασκάνιστος",63,0]},{"i":64,"r":["ἀβάσκανος",64,0]},{"i":65,"r":["ἀβάσκαντος",65,0]},{"i":66,"r":["ἀβάστακτος",66,0]},{"i":67,"r":["ἄβαστον",67,0]},{"i":68,"r":["ἀβατόομαι",68,0]},{"i":69,"r":["ἄβατος",69,0]},{"i":70,"r":["ἀβαφής",70,0]},{"i":71,"r":["ἄβδελον",71,0]},{"i":72,"r":["ἀβδέλυκτος",72,0]},{"i":73,"r":["Ἀβδηρίτης",73,0]},{"i":74,"r":["ἄβδης",74,0]},{"i":75,"r":["ἀβέβαιος",75,0]},{"i":76,"r":["ἀβεβαιότης",76,0]},{"i":77,"r":["ἀβέβηλος",77,0]},{"i":78,"r":["ἄβεις",78,0]},{"i":79,"r":["ἄβελλον",79,0]},{"i":80,"r":["ἀβελτέρειος",80,0]},{"i":81,"r":["ἀβελτερεύομαι",81,0]},{"i":82,"r":["ἀβελτερία",82,0]},{"i":83,"r":["ἀβελτεροκόκκυξ",83,0]},{"i":84,"r":["ἀβέλτερος",84,0]},{"i":85,"r":["ἀβέρβηλον",85,0]},{"i":86,"r":["ἀβηδών",86,0]},{"i":87,"r":["ἀβήρει",87,0]},{"i":88,"r":["ἀβηροῦσιν",88,0]},{"i":89,"r":["ἀβίαστος",89,0]},{"i":90,"r":["ἀβίβαστος",90,0]},{"i":91,"r":["ἀβίβλης",91,0]},{"i":92,"r":["ἄβιδα",92,0]},{"i":93,"r":["ἄβιν",93,0]},{"i":94,"r":["ἄβιος",94,0]},{"i":95,"r":["ἄβιος",95,0]},{"i":96,"r":["ἀβίοτος",96,0]},{"i":97,"r":["ἀβίυκτον",97,0]},{"i":98,"r":["ἀβιωτοποιός",98,0]},{"i":99,"r":["ἀβίωτος",99,0]},{"i":100,"r":["ἀβλάβεια",100,0]},{"i":101,"r":["ἀβλαβής",101,0]},{"i":102,"r":["ἀβλαβία",102,0]},{"i":103,"r":["ἀβλαβύνιον",103,0]},{"i":104,"r":["ἄβλαπτος",104,0]},{"i":105,"r":["ἄβλαροι",105,0]},{"i":106,"r":["ἀβλαστέω",106,0]},{"i":107,"r":["ἄβλαστος",107,0]},{"i":108,"r":["ἀβλάστητος",108,0]},{"i":109,"r":["ἄβλαυτος",109,0]},{"i":110,"r":["ἀβλεμής",110,0]},{"i":111,"r":["ἀβλεννής",111,0]},{"i":112,"r":["ἀβλεπτέω",112,0]},{"i":113,"r":["ἀβλεπτῆ",113,0]},{"i":114,"r":["ἀβλέπτημα",114,0]},{"i":115,"r":["ἄβλεπτος",115,0]},{"i":116,"r":["ἀβλέφαρος",116,0]},{"i":117,"r":["ἀβλεψία",117,0]},{"i":118,"r":["ἄβληρα",118,0]},{"i":119,"r":["ἀβλής",119,0]},{"i":120,"r":["ἀβλήτηρες",120,0]},{"i":121,"r":["ἄβλητος",121,0]},{"i":122,"r":["ἀβληχής",122,0]},{"i":123,"r":["ἀβληχρής",123,0]},{"i":124,"r":["ἀβληχρός",124,0]},{"i":125,"r":["ἀβλοπές",125,0]},{"i":126,"r":["ἀβοαί",126,0]},{"i":127,"r":["ἀβοατί",127,0]},{"i":128,"r":["ἀβοηθησία",128,0]},{"i":129,"r":["ἀβοήθητος",129,0]},{"i":130,"r":["ἀβοηθί",130,0]},{"i":131,"r":["ἀβοητί",131,0]},{"i":132,"r":["ἀβόητος",132,0]}]}
*/