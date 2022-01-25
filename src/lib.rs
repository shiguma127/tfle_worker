use std::{collections::HashMap};

use chrono::Utc;
use http::StatusCode;
use worker::*;
use sha256::{digest};
mod utils;

fn log_request(req: &Request) {
    console_log!(
        "{} - [{}], located at: {:?}, within: {}",
        Date::now().to_string(),
        req.path(),
        req.cf().coordinates().unwrap_or_default(),
        req.cf().region().unwrap_or("unknown region".into())
    );
}

#[event(fetch)]
pub async fn main(req: Request, env: Env) -> Result<Response> {
    log_request(&req);
    utils::set_panic_hook();
    let router = Router::new();
    router
        .get("/", |req, ctx| {
            let query_pairs: HashMap<_, _> = req.url()?.query_pairs().into_owned().collect();
            let user_answer = match query_pairs.get("answer") {
                Some(answer) => answer,
                None => return Response::error("Batrequest", StatusCode::BAD_REQUEST.as_u16()),
            };
            let user_answer:bool = match user_answer.parse() {
                Ok(answer) => answer,
                Err(_) => return Response::error("Batrequest", StatusCode::BAD_REQUEST.as_u16()),
            };
            let utc_date: chrono::Date<Utc> = Utc::today();
            let hash = digest(format!{"{}",utc_date});
            let bytes = hash.as_bytes();
            let answer = bytes[0] & 0x00000001 > 0;
            let res = answer == user_answer;
            let mut response  = Response::from_json(&res).unwrap();
            response.headers_mut().append("Access-Control-Allow-Origin", ctx.var("CLIENT_ORIGIN").unwrap().to_string().as_str())?;
            response.headers_mut().append("Access-Control-Allow-Methods","GET")?;
            Ok(response)
        })
        .run(req, env)
        .await
}
