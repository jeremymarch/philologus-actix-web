use super::*;
use crate::synopsis::polytonic_greek::hgk_compare_multiple_forms;
use hoplite_verbs_rs::*;
use sqlx::FromRow;
use std::sync::Arc;

#[derive(Debug, Deserialize, Clone)]
pub struct SynopsisRequest {
    // status:Option<String>, //tbd
    // unit:u32,
    // pp:Option<String>, //give either the pps
    // verb:Option<String>, //or give the verb_id
    person: String,
    number: String,
    // gender:Option<String>,
    // case:Option<String>,
}

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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SaverResults {
    pub given: String,
    pub correct: Option<String>,
    pub is_correct: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct SynopsisJsonResult {
    pub verb_id: usize,
    pub person: usize,
    pub number: usize,
    pub case: usize,
    pub gender: usize,
    pub unit: usize,
    pub pp: String,
    pub name: String,
    pub advisor: String,
    pub f: Vec<SaverResults>,
}

pub fn get_forms(
    verbs: &[Arc<HcGreekVerb>],
    verb_id: usize,
    person: &str,
    number: &str,
) -> Vec<Option<String>> {
    let mut forms = Vec::new();

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

    let numbers = match number {
        "plural" => [HcNumber::Plural],
        _ => [HcNumber::Singular],
    };
    let persons = match person {
        "1st" => [HcPerson::First],
        "2nd" => [HcPerson::Second],
        _ => [HcPerson::Third],
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
                        && (verbs[verb_id].pps[0] == "οἶδα" || verbs[verb_id].pps[0] == "σύνοιδα"))
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
                            verb: verbs[verb_id].clone(),
                            person: p,
                            number: n,
                            tense: t,
                            voice: v,
                            mood: m,
                            gender: None,
                            case: None,
                        };

                        if let Ok(f) = vf.get_form(false) {
                            forms.push(Some(f.last().unwrap().form.replace(" /", ",")))
                        } else {
                            forms.push(None)
                        }
                    }
                }
            }
        }
    }
    forms
}

pub async fn synopsis_json(
    (params, req): (web::Query<SynopsisRequest>, HttpRequest),
) -> Result<HttpResponse, AWError> {
    // let is_correct = hgk_compare_multiple_forms(&correct_answer, &info.answer.replace("---", "—"));
    let verbs = req.app_data::<Vec<Arc<HcGreekVerb>>>().unwrap();
    // let pp = "λω, λσω, ἔλῡσα, λέλυκα, λέλυμαι, ἐλύθην";
    // let verb = Arc::new(HcGreekVerb::from_string(1, pp, REGULAR, 0).unwrap());
    let verb_id: usize = 14;

    let forms = get_forms(verbs, verb_id, &params.person, &params.number);

    let mut res = Vec::<SaverResults>::new();
    for f in forms {
        res.push(SaverResults {
            given: f.unwrap_or("".to_string()),
            correct: None,
            is_correct: true,
        });
    }

    let res = SynopsisJsonResult {
        verb_id: 0,
        person: 0,
        number: 0,
        case: 0,
        gender: 0,
        unit: 0,
        pp: verbs[verb_id]
            .pps
            .iter()
            .map(|x| x.replace('/', " or ").replace("  ", " "))
            .collect::<Vec<_>>()
            .join(", "),
        name: "".to_string(),
        advisor: "".to_string(),
        f: res,
    };

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
    let verbs = req.app_data::<Vec<Arc<HcGreekVerb>>>().unwrap();

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

    let verb_id = 14;
    let correct_answers = get_forms(verbs, verb_id, &info.person, &info.number);
    let mut is_correct = Vec::new();
    // let is_correct = hgk_compare_multiple_forms(&correct_answer, &info.answer.replace("---", "—"));
    for (i, f) in info.r.iter().enumerate() {
        if let Some(a) = &correct_answers[i] {
            is_correct.push(hgk_compare_multiple_forms(a, &f.replace("---", "—")));
        } else {
            is_correct.push(true);
        }
    }

    let mut res_forms = Vec::<SaverResults>::new();
    for (n, i) in correct_answers.into_iter().enumerate() {
        res_forms.push(SaverResults {
            given: info.r[n].clone(),
            correct: i,
            is_correct: is_correct[n],
        });
    }

    let _ = greek_insert_synopsis(
        &db2.0,
        &info.into_inner(),
        time_stamp_ms,
        ip.as_str(),
        user_agent,
    )
    .await
    .map_err(map_sqlx_error)?;

    let res = SynopsisJsonResult {
        verb_id,
        person: 0,
        number: 0,
        case: 0,
        gender: 0,
        unit: 0,
        pp: verbs[verb_id]
            .pps
            .iter()
            .map(|x| x.replace('/', " or ").replace("  ", " "))
            .collect::<Vec<_>>()
            .join(", "),
        name: "".to_string(),
        advisor: "".to_string(),
        f: res_forms,
    };

    //Ok(HttpResponse::Ok().finish())
    //let res = 1;
    Ok(HttpResponse::Ok().json(res))
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

pub fn load_verbs(_path: &str) -> Vec<Arc<HcGreekVerb>> {
    let mut verbs = vec![];
    // if let Ok(pp_file) = File::open(path) {
    //     let pp_reader = BufReader::new(pp_file);
    //     for (idx, pp_line) in pp_reader.lines().enumerate() {
    //         if let Ok(line) = pp_line {
    //             if !line.starts_with('#') { //skip commented lines
    //                 verbs.push(Arc::new(HcGreekVerb::from_string_with_properties(idx as u32, &line).unwrap()));
    //             }
    //         }
    //     }
    // }
    verbs.push(Arc::new(
        HcGreekVerb::from_string_with_properties(0, "blank,blank,blank,blank,blank,blank % 0")
            .unwrap(),
    )); //so paideuw is at index 1
    let pp_lines = PPS.split('\n');
    for (idx, line) in pp_lines.enumerate() {
        if !line.starts_with('#') && !line.is_empty() {
            //skip commented lines
            //println!("line: {} {}", idx, line);
            verbs.push(Arc::new(
                HcGreekVerb::from_string_with_properties(idx as u32 + 1, line).unwrap(),
            ));
        }
    }

    verbs
}

pub static PPS: &str = r##"παιδεύω, παιδεύσω, ἐπαίδευσα, πεπαίδευκα, πεπαίδευμαι, ἐπαιδεύθην % 2
πέμπω, πέμψω, ἔπεμψα, πέπομφα, πέπεμμαι, ἐπέμφθην % 2
κελεύω, κελεύσω, ἐκέλευσα, κεκέλευκα, κεκέλευσμαι, ἐκελεύσθην % 2
λῡ́ω, λῡ́σω, ἔλῡσα, λέλυκα, λέλυμαι, ἐλύθην % 2
γράφω, γράψω, ἔγραψα, γέγραφα, γέγραμμαι, ἐγράφην % 3
θῡ́ω, θῡ́σω, ἔθῡσα, τέθυκα, τέθυμαι, ἐτύθην % 3
παύω, παύσω, ἔπαυσα, πέπαυκα, πέπαυμαι, ἐπαύθην % 3
φυλάττω, φυλάξω, ἐφύλαξα, πεφύλαχα, πεφύλαγμαι, ἐφυλάχθην % 3
διδάσκω, διδάξω, ἐδίδαξα, δεδίδαχα, δεδίδαγμαι, ἐδιδάχθην % 4
ἐθέλω, ἐθελήσω, ἠθέλησα, ἠθέληκα, —, — % 4
θάπτω, θάψω, ἔθαψα, —, τέθαμμαι, ἐτάφην % 4 % CONSONANT_STEM_PERFECT_PI
τάττω, τάξω, ἔταξα, τέταχα, τέταγμαι, ἐτάχθην % 4 % CONSONANT_STEM_PERFECT_GAMMA
ἄρχω, ἄρξω, ἦρξα, ἦρχα, ἦργμαι, ἤρχθην % 5 % CONSONANT_STEM_PERFECT_CHI
βλάπτω, βλάψω, ἔβλαψα, βέβλαφα, βέβλαμμαι, ἐβλάβην / ἐβλάφθην % 5 % CONSONANT_STEM_PERFECT_BETA
πείθω, πείσω, ἔπεισα, πέπεικα, πέπεισμαι, ἐπείσθην % 5
πρᾱ́ττω, πρᾱ́ξω, ἔπρᾱξα, πέπρᾱχα / πέπρᾱγα, πέπρᾱγμαι, ἐπρᾱ́χθην % 5 % CONSONANT_STEM_PERFECT_GAMMA
δουλεύω, δουλεύσω, ἐδούλευσα, δεδούλευκα, —, — % 6
κωλῡ́ω, κωλῡ́σω, ἐκώλῡσα, κεκώλῡκα, κεκώλῡμαι, ἐκωλῡ́θην % 6
πολῑτεύω, πολῑτεύσω, ἐπολῑ́τευσα, πεπολῑ́τευκα, πεπολῑ́τευμαι, ἐπολῑτεύθην % 6
χορεύω, χορεύσω, ἐχόρευσα, κεχόρευκα, κεχόρευμαι, ἐχορεύθην % 6
κλέπτω, κλέψω, ἔκλεψα, κέκλοφα, κέκλεμμαι, ἐκλάπην % 7 % CONSONANT_STEM_PERFECT_PI
λείπω, λείψω, ἔλιπον, λέλοιπα, λέλειμμαι, ἐλείφθην % 7 % CONSONANT_STEM_PERFECT_PI
σῴζω, σώσω, ἔσωσα, σέσωκα, σέσωσμαι / σέσωμαι, ἐσώθην % 7
ἄγω, ἄξω, ἤγαγον, ἦχα, ἦγμαι, ἤχθην % 8 % CONSONANT_STEM_PERFECT_GAMMA
ἥκω, ἥξω, —, —, —, — % 8
ἀδικέω, ἀδικήσω, ἠδίκησα, ἠδίκηκα, ἠδίκημαι, ἠδικήθην % 9
νῑκάω, νῑκήσω, ἐνῑ́κησα, νενῑ́κηκα, νενῑ́κημαι, ἐνῑκήθην % 9
ποιέω, ποιήσω, ἐποίησα, πεποίηκα, πεποίημαι, ἐποιήθην % 9
τῑμάω, τῑμήσω, ἐτῑ́μησα, τετῑ́μηκα, τετῑ́μημαι, ἐτῑμήθην % 9
ἀγγέλλω, ἀγγελῶ, ἤγγειλα, ἤγγελκα, ἤγγελμαι, ἠγγέλθην % 10 % CONSONANT_STEM_PERFECT_LAMBDA
ἀξιόω, ἀξιώσω, ἠξίωσα, ἠξίωκα, ἠξίωμαι, ἠξιώθην % 10
δηλόω, δηλώσω, ἐδήλωσα, δεδήλωκα, δεδήλωμαι, ἐδηλώθην % 10
καλέω, καλῶ, ἐκάλεσα, κέκληκα, κέκλημαι, ἐκλήθην % 10
μένω, μενῶ, ἔμεινα, μεμένηκα, —, — % 10
τελευτάω, τελευτήσω, ἐτελεύτησα, τετελεύτηκα, τετελεύτημαι, ἐτελευτήθην % 10
ἀκούω, ἀκούσομαι, ἤκουσα, ἀκήκοα, —, ἠκούσθην % 11
ἀποδέχομαι, ἀποδέξομαι, ἀπεδεξάμην, —, ἀποδέδεγμαι, — % 11 % CONSONANT_STEM_PERFECT_CHI PREFIXED
βάλλω, βαλῶ, ἔβαλον, βέβληκα, βέβλημαι, ἐβλήθην % 11
βούλομαι, βουλήσομαι, —, —, βεβούλημαι, ἐβουλήθην % 11
δέχομαι, δέξομαι, ἐδεξάμην, —, δέδεγμαι, — % 11 % CONSONANT_STEM_PERFECT_CHI
λαμβάνω, λήψομαι, ἔλαβον, εἴληφα, εἴλημμαι, ἐλήφθην % 11 % CONSONANT_STEM_PERFECT_BETA
πάσχω, πείσομαι, ἔπαθον, πέπονθα, —, — % 11
ἀνατίθημι, ἀναθήσω, ἀνέθηκα, ἀνατέθηκα, ἀνατέθειμαι, ἀνετέθην % 12 % PREFIXED
ἀποδίδωμι, ἀποδώσω, ἀπέδωκα, ἀποδέδωκα, ἀποδέδομαι, ἀπεδόθην % 12 % PREFIXED
ἀφίστημι, ἀποστήσω, ἀπέστησα / ἀπέστην, ἀφέστηκα, ἀφέσταμαι, ἀπεστάθην % 12 % PREFIXED
δίδωμι, δώσω, ἔδωκα, δέδωκα, δέδομαι, ἐδόθην % 12
ἵστημι, στήσω, ἔστησα / ἔστην, ἕστηκα, ἕσταμαι, ἐστάθην % 12
καθίστημι, καταστήσω, κατέστησα / κατέστην, καθέστηκα, καθέσταμαι, κατεστάθην % 12 % PREFIXED
καταλῡ́ω, καταλῡ́σω, κατέλῡσα, καταλέλυκα, καταλέλυμαι, κατελύθην % 12 % PREFIXED
τίθημι, θήσω, ἔθηκα, τέθηκα, τέθειμαι, ἐτέθην % 12
φιλέω, φιλήσω, ἐφίλησα, πεφίληκα, πεφίλημαι, ἐφιλήθην % 12
φοβέομαι, φοβήσομαι, —, —, πεφόβημαι, ἐφοβήθην % 12
γίγνομαι, γενήσομαι, ἐγενόμην, γέγονα, γεγένημαι, — % 13
ἔρχομαι, ἐλεύσομαι, ἦλθον, ἐλήλυθα, —, — % 13
μανθάνω, μαθήσομαι, ἔμαθον, μεμάθηκα, —, — % 13
μάχομαι, μαχοῦμαι, ἐμαχεσάμην, —, μεμάχημαι, — % 13
μεταδίδωμι, μεταδώσω, μετέδωκα, μεταδέδωκα, μεταδέδομαι, μετεδόθην % 13 % PREFIXED
μετανίσταμαι, μεταναστήσομαι, μετανέστην, μετανέστηκα, —, — % 13 % PREFIXED
μηχανάομαι, μηχανήσομαι, ἐμηχανησάμην, —, μεμηχάνημαι, — % 13
φεύγω, φεύξομαι, ἔφυγον, πέφευγα, —, — % 13
δείκνῡμι, δείξω, ἔδειξα, δέδειχα, δέδειγμαι, ἐδείχθην % 14
ἐπανίσταμαι, ἐπαναστήσομαι, ἐπανέστην, ἐπανέστηκα, —, —  % 14 % PREFIXED
ἐπιδείκνυμαι, ἐπιδείξομαι, ἐπεδειξάμην, —, ἐπιδέδειγμαι, — % 14 % PREFIXED
ἐρωτάω, ἐρωτήσω, ἠρώτησα, ἠρώτηκα, ἠρώτημαι, ἠρωτήθην % 14
λανθάνω, λήσω, ἔλαθον, λέληθα, —, — % 14
παραγίγνομαι, παραγενήσομαι, παρεγενόμην, παραγέγονα, παραγεγένημαι, — % 14 % PREFIXED
παραδίδωμι, παραδώσω, παρέδωκα, παραδέδωκα, παραδέδομαι, παρεδόθην % 14 % PREFIXED
παραμένω, παραμενῶ, παρέμεινα, παραμεμένηκα, —, — % 14 % PREFIXED
τυγχάνω, τεύξομαι, ἔτυχον, τετύχηκα, —, — % 14
ὑπακούω, ὑπακούσομαι, ὑπηκουσα, ὑπακήκοα, —, ὑπηκούσθην % 14 % PREFIXED
ὑπομένω, ὑπομενῶ, ὑπέμεινα, ὑπομεμένηκα, —, — % 14 % PREFIXED
φθάνω, φθήσομαι, ἔφθασα / ἔφθην, —, —, — % 14
χαίρω, χαιρήσω, —, κεχάρηκα, —, ἐχάρην % 14
αἱρέω, αἱρήσω, εἷλον, ᾕρηκα, ᾕρημαι, ᾑρέθην % 15
αἰσθάνομαι, αἰσθήσομαι, ᾐσθόμην, —, ᾔσθημαι, — % 15
διαφέρω, διοίσω, διήνεγκα / διήνεγκον, διενήνοχα, διενήνεγμαι, διηνέχθην % 15 % PREFIXED
εἰμί, ἔσομαι, —, —, —, — % 15
ἔστι(ν), ἔσται, —, —, —, — % 15
ἔξεστι(ν), ἐξέσται, —, —, —, — % 15
ἕπομαι, ἕψομαι, ἑσπόμην, —, —, — % 15
ὁράω, ὄψομαι, εἶδον, ἑόρᾱκα / ἑώρᾱκα, ἑώρᾱμαι / ὦμμαι, ὤφθην % 15 % CONSONANT_STEM_PERFECT_PI
συμφέρω, συνοίσω, συνήνεγκα / συνήνεγκον, συνενήνοχα, συνενήνεγμαι, συνηνέχθην % 15 % PREFIXED
φέρω, οἴσω, ἤνεγκα / ἤνεγκον, ἐνήνοχα, ἐνήνεγμαι, ἠνέχθην % 15
ἀναβαίνω, ἀναβήσομαι, ἀνέβην, ἀναβέβηκα, —, — % 16 % PREFIXED
βαίνω, -βήσομαι, -ἔβην, βέβηκα, —, — % 16
γιγνώσκω, γνώσομαι, ἔγνων, ἔγνωκα, ἔγνωσμαι, ἐγνώσθην % 16
ἐκπῑ́πτω, ἐκπεσοῦμαι, ἐξέπεσον, ἐκπέπτωκα, —, — % 16 % PREFIXED
λέγω, ἐρῶ / λέξω, εἶπον / ἔλεξα, εἴρηκα, εἴρημαι / λέλεγμαι, ἐλέχθην / ἐρρήθην % 16 % CONSONANT_STEM_PERFECT_GAMMA
νομίζω, νομιῶ, ἐνόμισα, νενόμικα, νενόμισμαι, ἐνομίσθην % 16
πῑ́πτω, πεσοῦμαι, ἔπεσον, πέπτωκα, —, — % 16
προδίδωμι, προδώσω, προέδωκα / προύδωκα, προδέδωκα, προδέδομαι, προεδόθην / προυδόθην % 16 % PREFIXED
φημί, φήσω, ἔφησα, —, —, — % 16
ἁμαρτάνω, ἁμαρτήσομαι, ἥμαρτον, ἡμάρτηκα, ἡμάρτημαι, ἡμαρτήθην % 17
δοκέω, δόξω, ἔδοξα, —, δέδογμαι, -ἐδόχθην % 17
δύναμαι, δυνήσομαι, —, —, δεδύνημαι, ἐδυνήθην % 17
εἶμι, —, —, —, —, — % 17
ἐλαύνω, ἐλῶ, ἤλασα, -ἐλήλακα, ἐλήλαμαι, ἠλάθην % 17
ἐπίσταμαι, ἐπιστήσομαι, —, —, —, ἠπιστήθην % 17
ἔχω, ἕξω / σχήσω, ἔσχον, ἔσχηκα, -ἔσχημαι, — % 17
ἀποθνῄσκω, ἀποθανοῦμαι, ἀπέθανον, τέθνηκα, —, — % 18 % PREFIXED
ἀποκτείνω, ἀποκτενῶ, ἀπέκτεινα, ἀπέκτονα, —, — % 18 % PREFIXED
ἀφῑ́ημι, ἀφήσω, ἀφῆκα, ἀφεῖκα, ἀφεῖμαι, ἀφείθην % 18 % PREFIXED
βουλεύω, βουλεύσω, ἐβούλευσα, βεβούλευκα, βεβούλευμαι, ἐβουλεύθην % 18
ἐπιβουλεύω, ἐπιβουλεύσω, ἐπεβούλευσα, ἐπιβεβούλευκα, ἐπιβεβούλευμαι, ἐπεβουλεύθην % 18 % PREFIXED
ζητέω, ζητήσω, ἐζήτησα, ἐζήτηκα, —, ἐζητήθην % 18
ῑ̔́ημι, -ἥσω, -ἧκα, -εἷκα, -εἷμαι, -εἵθην % 18
μέλλω, μελλήσω, ἐμέλλησα, —, —, — % 18
πιστεύω, πιστεύσω, ἐπίστευσα, πεπίστευκα, πεπίστευμαι, ἐπιστεύθην % 18
συμβουλεύω, συμβουλεύσω, συνεβούλευσα, συμβεβούλευκα, συμβεβούλευμαι, συνεβουλεύθην % 18 % PREFIXED
συνῑ́ημι, συνήσω, συνῆκα, συνεῖκα, συνεῖμαι, συνείθην % 18 % PREFIXED
αἰσχῡ́νομαι, αἰσχυνοῦμαι, —, —, ᾔσχυμμαι, ᾐσχύνθην % 19 % CONSONANT_STEM_PERFECT_NU
ἀποκρῑ́νομαι, ἀποκρινοῦμαι, ἀπεκρῑνάμην, —, ἀποκέκριμαι, — % 19
ἀπόλλῡμι, ἀπολῶ, ἀπώλεσα / ἀπωλόμην, ἀπολώλεκα / ἀπόλωλα, —, — % 19
—, ἀνερήσομαι, ἀνηρόμην, —, —, — % 19
—, ἐρήσομαι, ἠρόμην, —, —, — % 19
εὑρίσκω, εὑρήσω, ηὗρον, ηὕρηκα, ηὕρημαι, ηὑρέθην % 19
ἡγέομαι, ἡγήσομαι, ἡγησάμην, —, ἥγημαι, ἡγήθην % 19
κρῑ́νω, κρινῶ, ἔκρῑνα, κέκρικα, κέκριμαι, ἐκρίθην % 19
οἶδα, εἴσομαι, —, —, —, — % 19
σύνοιδα, συνείσομαι, —, —, —, — % 19
ἀφικνέομαι, ἀφίξομαι, ἀφῑκόμην, —, ἀφῖγμαι, — % 20 % PREFIXED
δεῖ, δεήσει, ἐδέησε(ν), —, —, — % 20
κεῖμαι, κείσομαι, —, —, —, — % 20
πυνθάνομαι, πεύσομαι, ἐπυθόμην, —, πέπυσμαι, — % 20
τρέπω, τρέψω, ἔτρεψα / ἐτραπόμην, τέτροφα, τέτραμμαι, ἐτράπην / ἐτρέφθην % 20 % CONSONANT_STEM_PERFECT_PI
φαίνω, φανῶ, ἔφηνα, πέφηνα, πέφασμαι, ἐφάνην % 20 % CONSONANT_STEM_PERFECT_NU
χρή, χρῆσται, —, —, —, — % 20
"##;
