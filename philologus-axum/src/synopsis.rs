use super::*;
use axum::response::Html;
use chrono::LocalResult;
use hoplite_verbs_rs::*;
use sqlx::FromRow;
use std::sync::Arc;

// the synopsis list/result/saver routes require the log db (env PHILOLOGUS_LOG_DB_PATH)
fn get_update_db(state: &AppState) -> Result<&SqlitePool, PhilologusError> {
    state
        .update_db
        .as_ref()
        .map(|db| &db.0)
        .ok_or_else(|| PhilologusError {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            name: "update db error".to_string(),
            error: "PHILOLOGUS_LOG_DB_PATH is not set".to_string(),
        })
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
    pub case: Option<usize>,
    pub gender: Option<usize>,
    pub unit: usize,
    pub pp: String,
    pub name: String,
    pub advisor: String,
    pub f: Vec<SaverResults>,
}

// verb ids come from the client, so reject any that are out of range
fn check_verb_id(verbs: &[Arc<HcGreekVerb>], verb_id: usize) -> Result<(), PhilologusError> {
    if verb_id < verbs.len() {
        Ok(())
    } else {
        Err(PhilologusError {
            code: StatusCode::BAD_REQUEST,
            name: "synopsis error".to_string(),
            error: format!("invalid verb id: {}", verb_id),
        })
    }
}

pub fn get_forms(
    verbs: &[Arc<HcGreekVerb>],
    verb_id: usize,
    person: usize,
    number: usize,
    case: Option<usize>,
    gender: Option<usize>,
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
        1 => [HcNumber::Plural],
        _ => [HcNumber::Singular],
    };
    let persons = match person {
        0 => [HcPerson::First],
        1 => [HcPerson::Second],
        _ => [HcPerson::Third],
    };

    let case_value = match case {
        Some(0) => Some(HcCase::Nominative),
        Some(1) => Some(HcCase::Genitive),
        Some(2) => Some(HcCase::Dative),
        Some(3) => Some(HcCase::Accusative),
        Some(4) => Some(HcCase::Vocative),
        _ => None,
    };

    let gender_value = match gender {
        Some(0) => Some(HcGender::Masculine),
        Some(1) => Some(HcGender::Feminine),
        Some(2) => Some(HcGender::Neuter),
        _ => None,
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
                            person: if m == HcMood::Infinitive || m == HcMood::Participle {
                                None
                            } else {
                                Some(p)
                            },
                            number: if m == HcMood::Infinitive {
                                None
                            } else {
                                Some(n)
                            },
                            tense: t,
                            voice: v,
                            mood: m,
                            gender: if m == HcMood::Participle {
                                gender_value
                            } else {
                                None
                            },
                            case: if m == HcMood::Participle {
                                case_value
                            } else {
                                None
                            },
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
    State(state): State<AppState>,
    Json(params): Json<SynopsisSaverRequest>,
) -> Result<Response, PhilologusError> {
    // let is_correct = hgk_compare_multiple_forms(&correct_answer, &info.answer.replace("---", "—"));
    let verbs = &state.verbs;
    // let pp = "λω, λσω, ἔλῡσα, λέλυκα, λέλυμαι, ἐλύθην";
    // let verb = Arc::new(HcGreekVerb::from_string(1, pp, REGULAR, 0).unwrap());
    let verb_id: usize = params.verb;
    check_verb_id(verbs, verb_id)?;

    let forms = get_forms(
        verbs,
        verb_id,
        params.person,
        params.number,
        params.ptccase,
        params.ptcgender,
    );

    let mut res = Vec::<SaverResults>::new();
    for f in forms {
        res.push(SaverResults {
            given: f.unwrap_or("".to_string()),
            correct: None,
            is_correct: true,
        });
    }

    let res = SynopsisJsonResult {
        verb_id: params.verb,
        person: params.person,
        number: params.number,
        case: params.ptccase,
        gender: params.ptcgender,
        unit: params.unit,
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

    Ok(Json(res).into_response())
}

pub async fn greek_synopsis_list(
    State(state): State<AppState>,
) -> Result<Response, PhilologusError> {
    let db2 = get_update_db(&state)?;

    let list = greek_get_synopsis_list(db2).await.map_err(map_sqlx_error)?;

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

    Ok(Html(res).into_response())
}

pub async fn greek_synopsis_result(
    State(state): State<AppState>,
    Query(info): Query<SynopsisResultRequest>,
) -> Result<Response, PhilologusError> {
    let db2 = get_update_db(&state)?;

    let list = greek_get_synopsis_result(db2, info.id)
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

    Ok(Html(res).into_response())
}

pub async fn greek_synopsis_saver(
    State(state): State<AppState>,
    parts: Parts,
    Json(info): Json<SynopsisSaverRequest>,
) -> Result<Response, PhilologusError> {
    let db2 = get_update_db(&state)?;
    let verbs = &state.verbs;

    let time_stamp = SystemTime::now().duration_since(UNIX_EPOCH);
    let time_stamp_ms = if let Ok(time_stamp) = time_stamp {
        time_stamp.as_millis()
    } else {
        0
    };
    let user_agent = get_user_agent(&parts.headers).unwrap_or("");
    let ip = get_ip(&parts);

    let verb_id = info.verb;
    check_verb_id(verbs, verb_id)?;
    let correct_answers = get_forms(
        verbs,
        verb_id,
        info.person,
        info.number,
        info.ptccase,
        info.ptcgender,
    );

    // missing answers are treated as ""; extra answers are ignored
    let mut res_forms = Vec::<SaverResults>::new();
    for (n, correct) in correct_answers.into_iter().enumerate() {
        let given = info.r.get(n).cloned().unwrap_or_default();
        let is_correct = match &correct {
            Some(a) => hgk_compare_multiple_forms(a, &given.replace("---", "—"), true),
            None => true,
        };
        res_forms.push(SaverResults {
            given,
            correct,
            is_correct,
        });
    }

    let res = SynopsisJsonResult {
        verb_id,
        person: info.person,
        number: info.number,
        case: info.ptccase,
        gender: info.ptcgender,
        unit: info.unit,
        pp: info.pp.clone(),
        // pp: verbs[verb_id]
        //     .pps
        //     .iter()
        //     .map(|x| x.replace('/', " or ").replace("  ", " "))
        //     .collect::<Vec<_>>()
        //     .join(", "),
        name: info.sname.clone(),
        advisor: info.advisor.clone(),
        f: res_forms,
    };

    let _ = greek_insert_synopsis(db2, &info, time_stamp_ms, ip.as_str(), user_agent)
        .await
        .map_err(map_sqlx_error)?;

    Ok(Json(res).into_response())
}

pub async fn cetest() -> Html<&'static str> {
    Html(include_str!("../../philologus-actix/src/cetest.html"))
}

pub async fn latin_synopsis_list(
    State(state): State<AppState>,
) -> Result<Response, PhilologusError> {
    let db2 = get_update_db(&state)?;

    let list = latin_get_synopsis_list(db2).await.map_err(map_sqlx_error)?;

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

    Ok(Html(res).into_response())
}

pub async fn latin_synopsis_result(
    State(state): State<AppState>,
    Query(info): Query<SynopsisResultRequest>,
) -> Result<Response, PhilologusError> {
    let db2 = get_update_db(&state)?;

    let list = latin_get_synopsis_result(db2, info.id)
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

    Ok(Html(res).into_response())
}

pub async fn latin_synopsis_saver(
    State(state): State<AppState>,
    parts: Parts,
    Json(info): Json<SynopsisSaverRequest>,
) -> Result<Response, PhilologusError> {
    let db2 = get_update_db(&state)?;

    let time_stamp = SystemTime::now().duration_since(UNIX_EPOCH);
    let time_stamp_ms = if let Ok(time_stamp) = time_stamp {
        time_stamp.as_millis()
    } else {
        0
    };
    let user_agent = get_user_agent(&parts.headers).unwrap_or("");
    let ip = get_ip(&parts);

    let _ = latin_insert_synopsis(db2, &info, time_stamp_ms, ip.as_str(), user_agent)
        .await
        .map_err(map_sqlx_error)?;

    Ok(Json(1).into_response())
}

pub async fn latin_synopsis() -> Html<String> {
    let mut template = include_str!("../../philologus-actix/src/latin-synopsis.html").to_string();

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

    Html(template)
}

static PPS: &str = include_str!("../../philologus-actix/src/pp.txt");

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

/*
CREATE TABLE latinsynopsisresults ( id INTEGER PRIMARY KEY AUTOINCREMENT, updated INTEGER NOT NULL, sname TEXT NOT NULL,
    advisor TEXT NOT NULL, sgiday INTEGER NOT NULL, selectedverb TEXT NOT NULL, pp TEXT NOT NULL, verbnumber TEXT NOT NULL,
    verbperson TEXT NOT NULL, verbptcgender TEXT NOT NULL, verbptcnumber TEXT NOT NULL, verbptccase TEXT NOT NULL, ip TEXT NOT NULL,
    ua TEXT NOT NULL, status INTEGER NOT NULL,
    f0 TEXT NOT NULL, f1 TEXT NOT NULL, f2 TEXT NOT NULL, f3 TEXT NOT NULL, f4 TEXT NOT NULL, f5 TEXT NOT NULL, f6 TEXT NOT NULL, f7 TEXT NOT NULL,
    f8 TEXT NOT NULL, f9 TEXT NOT NULL, f10 TEXT NOT NULL, f11 TEXT NOT NULL, f12 TEXT NOT NULL, f13 TEXT NOT NULL, f14 TEXT NOT NULL,
    f15 TEXT NOT NULL, f16 TEXT NOT NULL, f17 TEXT NOT NULL, f18 TEXT NOT NULL, f19 TEXT NOT NULL, f20 TEXT NOT NULL, f21 TEXT NOT NULL,
    f22 TEXT NOT NULL, f23 TEXT NOT NULL, f24 TEXT NOT NULL, f25 TEXT NOT NULL, f26 TEXT NOT NULL, f27 TEXT NOT NULL, f28 TEXT NOT NULL,
    f29 TEXT NOT NULL, f30 TEXT NOT NULL, f31 TEXT NOT NULL, f32 TEXT NOT NULL, f33 TEXT NOT NULL, f34 TEXT NOT NULL, f35 TEXT NOT NULL);


CREATE TABLE greeksynopsisresults ( id INTEGER PRIMARY KEY AUTOINCREMENT, updated INTEGER NOT NULL, sname TEXT NOT NULL, advisor TEXT NOT NULL, sgiday INTEGER NOT NULL, selectedverb TEXT NOT NULL, pp TEXT NOT NULL, verbnumber TEXT NOT NULL, verbperson TEXT NOT NULL, verbptcgender TEXT NOT NULL, verbptcnumber TEXT NOT NULL, verbptccase TEXT NOT NULL, ip TEXT NOT NULL, ua TEXT NOT NULL, status INTEGER NOT NULL, f0 TEXT NOT NULL, f1 TEXT NOT NULL, f2 TEXT NOT NULL, f3 TEXT NOT NULL, f4 TEXT NOT NULL, f5 TEXT NOT NULL, f6 TEXT NOT NULL, f7 TEXT NOT NULL, f8 TEXT NOT NULL, f9 TEXT NOT NULL, f10 TEXT NOT NULL, f11 TEXT NOT NULL, f12 TEXT NOT NULL, f13 TEXT NOT NULL, f14 TEXT NOT NULL, f15 TEXT NOT NULL, f16 TEXT NOT NULL, f17 TEXT NOT NULL, f18 TEXT NOT NULL, f19 TEXT NOT NULL, f20 TEXT NOT NULL, f21 TEXT NOT NULL, f22 TEXT NOT NULL, f23 TEXT NOT NULL, f24 TEXT NOT NULL, f25 TEXT NOT NULL, f26 TEXT NOT NULL, f27 TEXT NOT NULL, f28 TEXT NOT NULL, f29 TEXT NOT NULL, f30 TEXT NOT NULL, f31 TEXT NOT NULL, f32 TEXT NOT NULL, f33 TEXT NOT NULL, f34 TEXT NOT NULL, f35 TEXT NOT NULL, f36 TEXT NOT NULL, f37 TEXT NOT NULL, f38 TEXT NOT NULL, f39 TEXT NOT NULL, f40 TEXT NOT NULL, f41 TEXT NOT NULL, f42 TEXT NOT NULL, f43 TEXT NOT NULL, f44 TEXT NOT NULL, f45 TEXT NOT NULL, f46 TEXT NOT NULL, f47 TEXT NOT NULL, f48 TEXT NOT NULL, f49 TEXT NOT NULL, f50 TEXT NOT NULL, f51 TEXT NOT NULL, f52 TEXT NOT NULL, f53 TEXT NOT NULL, f54 TEXT NOT NULL, f55 TEXT NOT NULL, f56 TEXT NOT NULL, f57 TEXT NOT NULL, f58 TEXT NOT NULL, f59 TEXT NOT NULL, f60 TEXT NOT NULL, f61 TEXT NOT NULL, f62 TEXT NOT NULL);
*/
