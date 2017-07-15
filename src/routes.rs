use iron::{IronResult, Plugin, Request, Response, status, headers};
use iron::mime::{Mime, TopLevel, SubLevel};
use levenshtein::levenshtein;
use serde_json;
use urlencoded::UrlEncodedQuery;

use models;
use statics::*;

pub fn index(_: &mut Request) -> IronResult<Response> {
    let mut resp = Response::with((status::Ok, INDEXHTML));
    resp.headers.set(headers::ContentType(Mime(TopLevel::Text, SubLevel::Html, Vec::new())));
    Ok(resp)
}

pub fn suggestions(req: &mut Request) -> IronResult<Response> {
    let resptext = match req.get_ref::<UrlEncodedQuery>() {
        Ok(ref gets) => {
            if let Some(gqf) = gets.get("q").and_then(|gq| gq.first()).map(|gq| gq.to_lowercase()) {
                let indices = SUFFIXTABLE.positions(gqf.as_str());

                let mut result = Vec::new();
                let mut matchs = Vec::new();
                if !indices.is_empty() {
                    let score = 1. / indices.len() as f64;
                    for &idx32 in indices {
                        let idx = idx32 as usize;
                        let geodata_idx = match SUFFIXINDICES.binary_search(&idx) {
                            Ok(x) => x,
                            Err(x) => x-1,
                        };
                        let record = &GEODATA[geodata_idx];
                        matchs.push((record, levenshtein(record.name.as_str(), gqf.as_str())));
                    }
                    matchs.sort_by(|&(_, dista), &(_, distb)| dista.cmp(&distb));

                    for (data, _) in matchs.into_iter().take(20) {
                        result.push(models::Suggestion {
                            name: format!("{}, {}", data.name, data.country),
                            latitude: data.lat,
                            longitude: data.long,
                            score: score,
                        });
                    }
                }

                serde_json::to_string(&models::Suggestions { suggestions: result }).unwrap()
            } else {
                String::from("{err:\"Missing parameter: q\"}")
            }
        },
        Err(ref e) => serde_json::to_string(&models::Error { err: format!("Error: {:?}", e) }).unwrap(),
    };
    let mut resp = Response::with((status::Ok, resptext));
    resp.headers.set(headers::ContentType(Mime(TopLevel::Application, SubLevel::Json, Vec::new())));
    Ok(resp)
}
