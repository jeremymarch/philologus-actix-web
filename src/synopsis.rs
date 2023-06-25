use super::*;

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
    </head>
    <body><table>"#,
    );
    for l in list {
        let d = UNIX_EPOCH + Duration::from_millis(l.1.try_into().unwrap());
        let datetime = DateTime::<Local>::from(d);
        let timestamp_str = datetime.format("%Y-%m-%d %H:%M:%S").to_string();

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

        res.push_str(format!("<tr><td><a href='greek-synopsis-result?id={}'>{}</a></td><td>{}</td><td>{}</td><td>{}</td></tr>", l.0, timestamp_str, l.2, l.3,l.4).as_str());
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

pub async fn greek_synopsis(_req: HttpRequest) -> Result<HttpResponse, AWError> {
    let mut template = include_str!("greek-synopsis.html").to_string();

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
        let d = UNIX_EPOCH + Duration::from_millis(l.1.try_into().unwrap());
        let datetime = DateTime::<Local>::from(d);
        let timestamp_str = datetime.format("%Y-%m-%d %H:%M:%S").to_string();

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
    </head>
    <body><table class='synTable'>"#,
    );

    for l in list {
        let d = UNIX_EPOCH + Duration::from_millis(l.updated.try_into().unwrap());
        let datetime = DateTime::<Local>::from(d);
        let timestamp_str = datetime.format("%Y-%m-%d %H:%M:%S").to_string();

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
                "<tr><td class='label'>Date</td><td colspan='3'>{}</td></tr>",
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
