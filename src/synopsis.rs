use super::*;
use hoplite_verbs_rs::*;
use sqlx::FromRow;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct LatinSynopsisResult {
    pub id: i64,
    pub updated: i64,
    pub sname: String,
    pub advisor: String,
    pub sgiday: i64,
    pub selectedverb: String,
    pub pp: String,
    pub verbnumber: String,
    pub verbperson: String,
    pub verbptcgender: String,
    pub verbptcnumber: String,
    pub verbptccase: String,
    pub ip: String,
    pub ua: String,
    pub status: i64,
    pub f0: String,
    pub f1: String,
    pub f2: String,
    pub f3: String,
    pub f4: String,
    pub f5: String,
    pub f6: String,
    pub f7: String,
    pub f8: String,
    pub f9: String,
    pub f10: String,
    pub f11: String,
    pub f12: String,
    pub f13: String,
    pub f14: String,
    pub f15: String,
    pub f16: String,
    pub f17: String,
    pub f18: String,
    pub f19: String,
    pub f20: String,
    pub f21: String,
    pub f22: String,
    pub f23: String,
    pub f24: String,
    pub f25: String,
    pub f26: String,
    pub f27: String,
    pub f28: String,
    pub f29: String,
    pub f30: String,
    pub f31: String,
    pub f32: String,
    pub f33: String,
    pub f34: String,
    pub f35: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct GreekSynopsisResult {
    pub id: i64,
    pub updated: i64,
    pub sname: String,
    pub advisor: String,
    pub sgiday: i64,
    pub selectedverb: String,
    pub pp: String,
    pub verbnumber: String,
    pub verbperson: String,
    pub verbptcgender: String,
    pub verbptcnumber: String,
    pub verbptccase: String,
    pub ip: String,
    pub ua: String,
    pub status: i64,
    pub f0: String,
    pub f1: String,
    pub f2: String,
    pub f3: String,
    pub f4: String,
    pub f5: String,
    pub f6: String,
    pub f7: String,
    pub f8: String,
    pub f9: String,
    pub f10: String,
    pub f11: String,
    pub f12: String,
    pub f13: String,
    pub f14: String,
    pub f15: String,
    pub f16: String,
    pub f17: String,
    pub f18: String,
    pub f19: String,
    pub f20: String,
    pub f21: String,
    pub f22: String,
    pub f23: String,
    pub f24: String,
    pub f25: String,
    pub f26: String,
    pub f27: String,
    pub f28: String,
    pub f29: String,
    pub f30: String,
    pub f31: String,
    pub f32: String,
    pub f33: String,
    pub f34: String,
    pub f35: String,
    pub f36: String,
    pub f37: String,
    pub f38: String,
    pub f39: String,
    pub f40: String,
    pub f41: String,
    pub f42: String,
    pub f43: String,
    pub f44: String,
    pub f45: String,
    pub f46: String,
    pub f47: String,
    pub f48: String,
    pub f49: String,
    pub f50: String,
    pub f51: String,
    pub f52: String,
    pub f53: String,
    pub f54: String,
    pub f55: String,
    pub f56: String,
    pub f57: String,
    pub f58: String,
    pub f59: String,
    pub f60: String,
    pub f61: String,
    pub f62: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct SynopsisJsonResult {
    pub pp: String,
    pub f: Vec<Option<String>>,
}

pub async fn synopsis_json(_req: HttpRequest) -> Result<HttpResponse, AWError> {
    let pp = "λω, λσω, ἔλῡσα, λέλυκα, λέλυμαι, ἐλύθην";
    let verb = Arc::new(HcGreekVerb::from_string(1, pp, REGULAR, 0).unwrap());

    let tenses = [
        HcTense::Present,
        HcTense::Imperfect,
        HcTense::Future,
        HcTense::Aorist,
        HcTense::Perfect,
        HcTense::Pluperfect,
    ];

    let voices = [HcVoice::Active, HcVoice::Middle, HcVoice::Passive];
    let moods = [
        HcMood::Indicative,
        HcMood::Subjunctive,
        HcMood::Optative,
        HcMood::Imperative,
        HcMood::Infinitive,
        HcMood::Participle,
    ];

    let numbers = [HcNumber::Singular /*, HcNumber::Plural*/];
    let persons = [/*HcPerson::First, HcPerson::Second,*/ HcPerson::Third];

    let mut res = SynopsisJsonResult {
        pp: pp.to_string(),
        f: Vec::new(),
    };

    for m in moods {
        for t in tenses {
            for v in voices {
                if ((m == HcMood::Subjunctive || m == HcMood::Optative || m == HcMood::Imperative)
                    && (t == HcTense::Imperfect
                        || t == HcTense::Perfect
                        || t == HcTense::Pluperfect))
                    || t == HcTense::Future && (m == HcMood::Subjunctive || m == HcMood::Imperative)
                {
                    // allow moods for oida, synoida
                    if !((m == HcMood::Subjunctive
                        || m == HcMood::Optative
                        || m == HcMood::Imperative)
                        && t == HcTense::Perfect
                        && v == HcVoice::Active
                        && (verb.pps[0] == "οἶδα" || verb.pps[0] == "σύνοιδα"))
                    {
                        continue;
                    }
                }

                if (m == HcMood::Infinitive || m == HcMood::Participle)
                    && (t == HcTense::Imperfect || t == HcTense::Pluperfect)
                {
                    continue;
                }

                for n in numbers {
                    for p in persons {
                        let vf = HcGreekVerbForm {
                            verb: verb.clone(),
                            person: p,
                            number: n,
                            tense: t,
                            voice: v,
                            mood: m,
                            gender: None,
                            case: None,
                        };

                        if let Ok(f) = vf.get_form(false) {
                            res.f.push(Some(f.last().unwrap().form.replace(" /", ",")))
                        } else {
                            res.f.push(None)
                        }
                    }
                }
            }
        }
    }
    Ok(HttpResponse::Ok().json(res))
}

pub async fn greek_synopsis_list(req: HttpRequest) -> Result<HttpResponse, AWError> {
    let db2 = req.app_data::<SqliteUpdatePool>().unwrap();

    let list = greek_get_synopsis_list(&db2.0)
        .await
        .map_err(map_sqlx_error)?;

    let mut res = String::from(
        r#"<!DOCTYPE html>
    <html>
    <head>
    <meta charset="UTF-8">
    <style nonce="2726c7f26c">
        .synlist { width: 600px;
            margin: 0px auto;
            border-collapse: collapse;
            font-size: 16pt;
            font-family:helvetica,arial;
        }
        .synlist td { padding: 3px; }
        .headerrow {border-bottom:1px solid black;font-weight:bold;}
    </style>
    </head>
    <body><table class='synlist'>
    <tr><td class='headerrow'>Date</td><td class='headerrow'>Name</td><td class='headerrow'>Advisor</td><td class='headerrow'>Verb</td></tr>"#,
    );
    for l in list {
        let eastern_daylight_tz = FixedOffset::west_opt(4 * 60 * 60).unwrap();
        let d = eastern_daylight_tz.timestamp_millis_opt(l.1);
        let timestamp_str = match d {
            LocalResult::Single(t) => t.format("%Y-%m-%d %H:%M:%S").to_string(),
            _ => "".to_string(),
        };

        res.push_str(format!("<tr><td><a href='greek-synopsis-result?id={}'>{}</a></td><td>{}</td><td>{}</td><td>{}</td></tr>", l.0, timestamp_str, l.2, l.3,l.4).as_str());
    }
    res.push_str("</table></body></html>");

    Ok(HttpResponse::Ok().content_type("text/html").body(res))
}

pub async fn greek_synopsis_result(
    (info, req): (web::Query<SynopsisResultRequest>, HttpRequest),
) -> Result<HttpResponse, AWError> {
    let db2 = req.app_data::<SqliteUpdatePool>().unwrap();

    let list = greek_get_synopsis_result(&db2.0, info.id)
        .await
        .map_err(map_sqlx_error)?;

    let mut res = String::from(
        r#"<!DOCTYPE html>
    <html lang='en'>
    <head>
    <meta charset="UTF-8">
    <style nonce="2726c7f26c">
    @font-face {
        font-family: 'WebNewAthenaUnicode';
        src: url('/newathu5_8.ttf') format('truetype');
      }  
    BODY {font-family:helvetica,arial}
    .synTable { min-width:800px; font-size:16pt; border-spacing:0px;border-collapse: collapse;margin: 0px auto; }
    .synTable td {padding:4px 5px;}
    .labelcol { width:15%;}
    .label {font-weight:bold; }
    .spacer { width:50px;}
    .majorlabelrow { border-top:1px solid black;font-weight:bold;}
    .greek { font-family: WebNewAthenaUnicode;}
    </style>
    <script nonce="2726c7f26c">
    function fixdate() {
        const ds = document.getElementById('submittedDate');
        const date = new Date(parseInt(ds.innerHTML));
        ds.innerHTML = date.toLocaleString(navigator.language);
    }
    // window.addEventListener('load', fixdate, false);
    </script>
    </head>
    <body><table class='synTable'>"#,
    );
    for l in list {
        let eastern_daylight_tz = FixedOffset::west_opt(4 * 60 * 60).unwrap();
        let d = eastern_daylight_tz.timestamp_millis_opt(l.updated);
        let timestamp_str = match d {
            LocalResult::Single(t) => t.format("%Y-%m-%d %H:%M:%S").to_string(),
            _ => "".to_string(),
        };
        res.push_str(
            format!(
                "<tr><td colspan='2' class='label'>Name</td><td colspan='3'>{}</td></tr>",
                l.sname
            )
            .as_str(),
        );
        res.push_str(
            format!(
                "<tr><td colspan='2' class='label'>Advisor</td><td colspan='3'>{}</td></tr>",
                l.advisor
            )
            .as_str(),
        );
        res.push_str(
            format!(
                "<tr><td colspan='2' class='label'>Date</td><td id='submittedDate' colspan='3'>{}</td></tr>",
                timestamp_str
            )
            .as_str(),
        );
        res.push_str(
            format!(
                "<tr><td colspan='2' class='label'>Pers., Num., Gen., Case</td><td colspan='3'>{}, {}, {}, {}</td></tr>",
                l.verbperson, l.verbnumber, l.verbptcgender, l.verbptccase
            )
            .as_str(),
        );
        //res.push_str("<tr><td colspan='5'>&nbsp;</td></tr>");
        res.push_str(
            format!(
                "<tr><td colspan='2' class='label'>Principal Parts</td><td colspan='3' class='greek'>{}</td></tr>",
                l.pp
            )
            .as_str(),
        );
        //res.push_str("<tr><td colspan='5'>&nbsp;</td></tr>");
        res.push_str("<tr><td colspan='2'>&nbsp;</td><td class='label' align='left'>Active</td><td class='label' align='left'>Middle</td><td class='label' align='left'>Passive</td></tr>");

        res.push_str("<tr><td colspan='5' class='majorlabelrow'>Indicative</td></tr>");
        res.push_str(format!("<tr><td class='spacer'>&nbsp;</td><td class='labelcol label'>{}</td><td class='greek'>{}</td><td class='greek'>{}</td><td class='greek'>{}</td></tr>", "Present",  l.f0, l.f1, l.f2).as_str());
        res.push_str(format!("<tr><td class='spacer'>&nbsp;</td><td class='labelcol label'>{}</td><td class='greek'>{}</td><td class='greek'>{}</td><td class='greek'>{}</td></tr>", "Imperfect",  l.f3, l.f4, l.f5).as_str());
        res.push_str(format!("<tr><td class='spacer'>&nbsp;</td><td class='labelcol label'>{}</td><td class='greek'>{}</td><td class='greek'>{}</td><td class='greek'>{}</td></tr>", "Future",  l.f6, l.f7, l.f8).as_str());
        res.push_str(format!("<tr><td class='spacer'>&nbsp;</td><td class='labelcol label'>{}</td><td class='greek'>{}</td><td class='greek'>{}</td><td class='greek'>{}</td></tr>", "Aorist",  l.f9, l.f10, l.f11).as_str());
        res.push_str(format!("<tr><td class='spacer'>&nbsp;</td><td class='labelcol label'>{}</td><td class='greek'>{}</td><td class='greek'>{}</td><td class='greek'>{}</td></tr>", "Perfect",  l.f12, l.f13, l.f14).as_str());
        res.push_str(format!("<tr><td class='spacer'>&nbsp;</td><td class='labelcol label'>{}</td><td class='greek'>{}</td><td class='greek'>{}</td><td class='greek'>{}</td></tr>", "Pluperfect",  l.f15, l.f16, l.f17).as_str());

        res.push_str("<tr><td colspan='5' class='majorlabelrow'>Subjunctive</td></tr>");
        res.push_str(format!("<tr><td class='spacer'>&nbsp;</td><td class='labelcol label'>{}</td><td class='greek'>{}</td><td class='greek'>{}</td><td class='greek'>{}</td></tr>", "Present",  l.f18, l.f19, l.f20).as_str());
        res.push_str(format!("<tr><td class='spacer'>&nbsp;</td><td class='labelcol label'>{}</td><td class='greek'>{}</td><td class='greek'>{}</td><td class='greek'>{}</td></tr>", "Aorist",  l.f21, l.f22, l.f23).as_str());

        res.push_str("<tr><td colspan='5' class='majorlabelrow'>Optative</td></tr>");
        res.push_str(format!("<tr><td class='spacer'>&nbsp;</td><td class='labelcol label'>{}</td><td class='greek'>{}</td><td class='greek'>{}</td><td class='greek'>{}</td></tr>", "Present",  l.f24, l.f25, l.f26).as_str());
        res.push_str(format!("<tr><td class='spacer'>&nbsp;</td><td class='labelcol label'>{}</td><td class='greek'>{}</td><td class='greek'>{}</td><td class='greek'>{}</td></tr>", "Future",  l.f27, l.f28, l.f29).as_str());
        res.push_str(format!("<tr><td class='spacer'>&nbsp;</td><td class='labelcol label'>{}</td><td class='greek'>{}</td><td class='greek'>{}</td><td class='greek'>{}</td></tr>", "Aorist",  l.f30, l.f31, l.f32).as_str());

        res.push_str("<tr><td colspan='5' class='majorlabelrow'>Imperative</td></tr>");
        res.push_str(format!("<tr><td class='spacer'>&nbsp;</td><td class='labelcol label'>{}</td><td class='greek'>{}</td><td class='greek'>{}</td><td class='greek'>{}</td></tr>", "Present",  l.f33, l.f34, l.f35).as_str());
        res.push_str(format!("<tr><td class='spacer'>&nbsp;</td><td class='labelcol label'>{}</td><td class='greek'>{}</td><td class='greek'>{}</td><td class='greek'>{}</td></tr>", "Aorist",  l.f36, l.f37, l.f38).as_str());

        res.push_str("<tr><td colspan='5' class='majorlabelrow'>Infinitive</td></tr>");
        res.push_str(format!("<tr><td class='spacer'>&nbsp;</td><td class='labelcol label'>{}</td><td class='greek'>{}</td><td class='greek'>{}</td><td class='greek'>{}</td></tr>", "Present",  l.f39, l.f40, l.f41).as_str());
        res.push_str(format!("<tr><td class='spacer'>&nbsp;</td><td class='labelcol label'>{}</td><td class='greek'>{}</td><td class='greek'>{}</td><td class='greek'>{}</td></tr>", "Future",  l.f42, l.f43, l.f44).as_str());
        res.push_str(format!("<tr><td class='spacer'>&nbsp;</td><td class='labelcol label'>{}</td><td class='greek'>{}</td><td class='greek'>{}</td><td class='greek'>{}</td></tr>", "Aorist",  l.f45, l.f46, l.f47).as_str());
        res.push_str(format!("<tr><td class='spacer'>&nbsp;</td><td class='labelcol label'>{}</td><td class='greek'>{}</td><td class='greek'>{}</td><td class='greek'>{}</td></tr>", "Perfect",  l.f48, l.f49, l.f50).as_str());

        res.push_str("<tr><td colspan='5' class='majorlabelrow'>Participle</td></tr>");
        res.push_str(format!("<tr><td class='spacer'>&nbsp;</td><td class='labelcol label'>{}</td><td class='greek'>{}</td><td class='greek'>{}</td><td class='greek'>{}</td></tr>", "Present",  l.f51, l.f52, l.f53).as_str());
        res.push_str(format!("<tr><td class='spacer'>&nbsp;</td><td class='labelcol label'>{}</td><td class='greek'>{}</td><td class='greek'>{}</td><td class='greek'>{}</td></tr>", "Future",  l.f54, l.f55, l.f56).as_str());
        res.push_str(format!("<tr><td class='spacer'>&nbsp;</td><td class='labelcol label'>{}</td><td class='greek'>{}</td><td class='greek'>{}</td><td class='greek'>{}</td></tr>", "Aorist",  l.f57, l.f58, l.f59).as_str());
        res.push_str(format!("<tr><td class='spacer'>&nbsp;</td><td class='labelcol label'>{}</td><td class='greek'>{}</td><td class='greek'>{}</td><td class='greek'>{}</td></tr>", "Perfect",  l.f60, l.f61, l.f62).as_str());
    }
    res.push_str("</table></body></html>");

    Ok(HttpResponse::Ok().content_type("text/html").body(res))
}

pub async fn greek_synopsis_saver(
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

    let _ = greek_insert_synopsis(
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

pub async fn cetest(_req: HttpRequest) -> Result<HttpResponse, AWError> {
    let template = include_str!("cetest.html").to_string();
    Ok(HttpResponse::Ok().content_type("text/html").body(template))
}

pub async fn greek_synopsis(_req: HttpRequest) -> Result<HttpResponse, AWError> {
    let mut template = include_str!("greek-synopsis.html").to_string();

    let mut rows = String::from("");
    let mut count = 0;
    let rowlabels = vec![
        "Present Indicative",
        "Imperfect Indicative",
        "Future Indicative",
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
use chrono::LocalResult;
pub async fn latin_synopsis_list(req: HttpRequest) -> Result<HttpResponse, AWError> {
    let db2 = req.app_data::<SqliteUpdatePool>().unwrap();

    let list = latin_get_synopsis_list(&db2.0)
        .await
        .map_err(map_sqlx_error)?;

    let mut res = String::from(
        r#"<!DOCTYPE html>
    <html>
    <head>
    <meta charset="UTF-8">
    <style nonce="2726c7f26c">
        .synlist { width: 600px;
            margin: 0px auto;
            border-collapse: collapse;
            font-size: 16pt;
            font-family:helvetica,arial;
        }
        .synlist td { padding: 3px; }
        .headerrow {border-bottom:1px solid black;font-weight:bold;}
    
    </style>
    </head>
    <body><table class='synlist'>
    <tr><td class='headerrow'>Date</td><td class='headerrow'>Name</td><td class='headerrow'>Advisor</td><td class='headerrow'>Verb</td></tr>"#,
    );
    for l in list {
        let eastern_daylight_tz = FixedOffset::west_opt(4 * 60 * 60).unwrap();
        let d = eastern_daylight_tz.timestamp_millis_opt(l.1);
        let timestamp_str = match d {
            LocalResult::Single(t) => t.format("%Y-%m-%d %H:%M:%S").to_string(),
            _ => "".to_string(),
        };

        res.push_str(format!("<tr><td><a href='latin-synopsis-result?id={}'>{}</a></td><td>{}</td><td>{}</td><td>{}</td></tr>", l.0, timestamp_str, l.2, l.3,l.4).as_str());
    }
    res.push_str("</table></body></html>");

    Ok(HttpResponse::Ok().content_type("text/html").body(res))
}

pub async fn latin_synopsis_result(
    (info, req): (web::Query<SynopsisResultRequest>, HttpRequest),
) -> Result<HttpResponse, AWError> {
    let db2 = req.app_data::<SqliteUpdatePool>().unwrap();

    let list = latin_get_synopsis_result(&db2.0, info.id)
        .await
        .map_err(map_sqlx_error)?;

    let mut res = String::from(
        r#"<!DOCTYPE html>
    <html lang='en'>
    <head>
    <meta charset="UTF-8">
    <style nonce="2726c7f26c">
    BODY {font-family:helvetica,arial}
    .synTable { min-width:800px; font-size:16pt; border-spacing:0px;border-collapse: collapse;margin: 0px auto; }
    .synTable td {padding:4px 5px;}
    .labelcol { width:25%;}
    .label {font-weight:bold; }
    .spacer { width:25%;}
    .majorlabelrow { border-top:1px solid black;}
    </style>
    <script nonce="2726c7f26c">
    function fixdate() {
        const ds = document.getElementById('submittedDate');
        const date = new Date(parseInt(ds.innerHTML));
        ds.innerHTML = date.toLocaleString(navigator.language);
    }
    // window.addEventListener('load', fixdate, false);
    </script>
    </head>
    <body><table class='synTable'>"#,
    );

    for l in list {
        let eastern_daylight_tz = FixedOffset::west_opt(4 * 60 * 60).unwrap();
        let d = eastern_daylight_tz.timestamp_millis_opt(l.updated);
        let timestamp_str = match d {
            LocalResult::Single(t) => t.format("%Y-%m-%d %H:%M:%S").to_string(),
            _ => "".to_string(),
        };

        res.push_str(
            format!(
                "<tr><td class='label'>Name</td><td colspan='3'>{}</td></tr>",
                l.sname
            )
            .as_str(),
        );
        res.push_str(
            format!(
                "<tr><td class='label'>Advisor</td><td colspan='3'>{}</td></tr>",
                l.advisor
            )
            .as_str(),
        );
        res.push_str(
            format!(
                "<tr><td class='label'>Date</td><td id='submittedDate' colspan='3'>{}</td></tr>",
                timestamp_str
            )
            .as_str(),
        );
        res.push_str(
            format!(
                "<tr><td class='label'>Pers. Num. Gen.</td><td colspan='3'>{}, {}, {}</td></tr>",
                l.verbperson, l.verbnumber, l.verbptcgender
            )
            .as_str(),
        );

        res.push_str("<tr><td colspan='4'>&nbsp;</td></tr>");
        res.push_str(
            format!(
                "<tr><td class='label'>Principal Parts</td><td colspan='3'>{}</td></tr>",
                l.pp
            )
            .as_str(),
        );
        res.push_str("<tr><td colspan='4'>&nbsp;</td></tr>");
        res.push_str("<tr><td class='spacer'><td></td></td><td class='label' align='left'>Active</td><td class='label' align='left'>Passive</td></tr>");

        res.push_str("<tr><td class='label majorlabelrow' colspan='4'>Indicative</td></tr>");
        res.push_str(format!("<tr><td class='spacer'></td><td class='labelcol label'>{}</td><td>{}</td><td>{}</td></tr>", "Present",  l.f0, l.f1).as_str());
        res.push_str(format!("<tr><td class='spacer'></td><td class='labelcol label'>{}</td><td>{}</td><td>{}</td></tr>", "Imperfect",  l.f2, l.f3).as_str());
        res.push_str(format!("<tr><td class='spacer'></td><td class='labelcol label'>{}</td><td>{}</td><td>{}</td></tr>", "Future",  l.f4, l.f5).as_str());
        res.push_str("<tr><td class='label majorlabelrow' colspan='4'>Subjunctive</td></tr>");
        res.push_str(format!("<tr><td class='spacer'></td><td class='labelcol label'>{}</td><td>{}</td><td>{}</td></tr>", "Present",  l.f6, l.f7).as_str());
        res.push_str(format!("<tr><td class='spacer'></td><td class='labelcol label'>{}</td><td>{}</td><td>{}</td></tr>", "Imperfect",  l.f8, l.f9).as_str());
        res.push_str("<tr><td class='label majorlabelrow' colspan='4'>Indicative</td></tr>");
        res.push_str(format!("<tr><td class='spacer'></td><td class='labelcol label'>{}</td><td>{}</td><td>{}</td></tr>", "Perfect",  l.f10, l.f11).as_str());
        res.push_str(format!("<tr><td class='spacer'></td><td class='labelcol label'>{}</td><td>{}</td><td>{}</td></tr>", "Pluperfect",  l.f12, l.f13).as_str());
        res.push_str(format!("<tr><td class='spacer'></td><td class='labelcol label'>{}</td><td>{}</td><td>{}</td></tr>", "Future Perfect",  l.f14, l.f15).as_str());
        res.push_str("<tr><td class='label majorlabelrow' colspan='4'>Subjunctive</td></tr>");
        res.push_str(format!("<tr><td class='spacer'></td><td class='labelcol label'>{}</td><td>{}</td><td>{}</td></tr>", "Perfect",  l.f16, l.f17).as_str());
        res.push_str(format!("<tr><td class='spacer'></td><td class='labelcol label'>{}</td><td>{}</td><td>{}</td></tr>", "Pluperfect",  l.f18, l.f19).as_str());
        res.push_str("<tr><td class='label majorlabelrow' colspan='4'>Participles</td></tr>");
        res.push_str(format!("<tr><td class='spacer'></td><td class='labelcol label'>{}</td><td>{}</td><td>{}</td></tr>", "Present",  l.f20, l.f21).as_str());
        res.push_str(format!("<tr><td class='spacer'></td><td class='labelcol label'>{}</td><td>{}</td><td>{}</td></tr>", "Perfect",  l.f22, l.f23).as_str());
        res.push_str(format!("<tr><td class='spacer'></td><td class='labelcol label'>{}</td><td>{}</td><td>{}</td></tr>", "Future",  l.f24, l.f25).as_str());
        res.push_str("<tr><td class='label majorlabelrow' colspan='4'>Infintives</td></tr>");
        res.push_str(format!("<tr><td class='spacer'></td><td class='labelcol label'>{}</td><td>{}</td><td>{}</td></tr>", "Present",  l.f26, l.f27).as_str());
        res.push_str(format!("<tr><td class='spacer'></td><td class='labelcol label'>{}</td><td>{}</td><td>{}</td></tr>", "Perfect",  l.f28, l.f29).as_str());
        res.push_str(format!("<tr><td class='spacer'></td><td class='labelcol label'>{}</td><td>{}</td><td>{}</td></tr>", "Future",  l.f30, l.f31).as_str());
        res.push_str("<tr><td class='label majorlabelrow' colspan='4'>Imperatives</td></tr>");
        res.push_str(format!("<tr><td class='spacer'></td><td class='labelcol label'>{}</td><td>{}</td><td>{}</td></tr>", "Singular",  l.f32, l.f33).as_str());
        res.push_str(format!("<tr><td class='spacer'></td><td class='labelcol label'>{}</td><td>{}</td><td>{}</td></tr>", "Plural",  l.f34, l.f35).as_str());
    }
    res.push_str("</table></body></html>");

    Ok(HttpResponse::Ok().content_type("text/html").body(res))
}

pub async fn latin_synopsis_saver(
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

    let _ = latin_insert_synopsis(
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

pub async fn latin_synopsis(_req: HttpRequest) -> Result<HttpResponse, AWError> {
    let mut template = include_str!("latin-synopsis.html").to_string();

    let mut rows = String::from("");
    let mut count = 0;
    let rowlabels = vec![
        "Present",
        "Imperfect",
        "Future",
        "Present",
        "Imperfect",
        "Perfect",
        "Pluperfect",
        "Future Perfect ",
        "Perfect",
        "Pluperfect",
        "Present",
        "Perfect",
        "Future",
        "Present",
        "Perfect",
        "Future",
        "Singular",
        "Plural",
    ];
    let voices = vec!["Active", "Passive"];
    for l in rowlabels {
        if count == 0 {
            rows.push_str("<tr><td colspan='3' class='majorrowheader'>Indicative</td></tr>");
        } else if count == 6 {
            rows.push_str("<tr><td colspan='3' class='majorrowheader'>Subjunctive</td></tr>");
        } else if count == 10 {
            rows.push_str("<tr><td colspan='3' class='majorrowheader'>Indicative</td></tr>");
        } else if count == 16 {
            rows.push_str("<tr><td colspan='3' class='majorrowheader'>Subjunctive</td></tr>");
        } else if count == 20 {
            rows.push_str("<tr><td colspan='3' class='majorrowheader'>Participles</td></tr>");
        } else if count == 26 {
            rows.push_str("<tr><td colspan='3' class='majorrowheader'>Infinitives</td></tr>");
        } else if count == 32 {
            rows.push_str("<tr><td colspan='3' class='majorrowheader'>Imperatives</td></tr>");
        }
        rows.push_str(
            format!(
                r#"<tr class="{}"><td class='formcelllabel'>{}</td>"#,
                l.to_lowercase(),
                l
            )
            .as_str(),
        );
        for v in &voices {
            rows.push_str(format!(
            r#"<td class="formcell {}">
                <div class="formcellInner">
                <input type="text" id="gkform{}" class="gkinput formcellinput" spellcheck="false" autocapitalize="off" autocomplete="off" {}/>
                </div>
            </td>"#,
            v.to_lowercase(), count, if count == 21 || count == 22 {"disabled"} else {""}).as_str());
            count += 1;
        }
        rows.push_str("</tr>");
    }

    template = template.replacen("%rows%", &rows, 1);

    Ok(HttpResponse::Ok().content_type("text/html").body(template))
}
