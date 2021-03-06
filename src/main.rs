use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use serde::{Serialize, Deserialize};
use reqwest::header::{AUTHORIZATION, HeaderValue, };
use oauth2::{ TokenResponse, };

mod oauth;
mod api;

#[derive(Serialize, Deserialize, Debug)]
struct Creds {
    client_id: String,
    client_secret: String,
}

// return type is a Result type with a Unit Ok, or any type that implements the Error trait object methods (boxed because trait object)
fn main() -> Result<(), Box<dyn std::error::Error>> {

    // read json file
    let file = File::open("./src/basecamp.json")?;
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents)?;
    let json: Creds = serde_json::from_str(&contents).expect("JSON was not well-formatted");
    // println!("json {:?}", json);

    let at = oauth::get_auth_token(json.client_id, json.client_secret)?;
    // println!("main at {:?}", at);

    let client = build_client(at.access_token())?;

    let answers_body = api::collect_answer_data(&client)?;

    println!("length of answers {}", answers_body.len());
    // now I have all of the data... now what?

    let (yes_counter, no_counter) = count_ans(&answers_body);
    println!("yes: {}, no: {}", yes_counter, no_counter);

    Ok(())
}

fn count_ans (answers: &Vec<api::Answer>) -> (u32, u32) {
    // remove the div tag
    answers.iter().map(|ans| ans.content.as_str().strip_prefix("<div>").unwrap().strip_suffix("</div>").unwrap())
    // take the first word, remove the trailing punctuation
    .map(|ans| ans.split_ascii_whitespace().next().unwrap().trim_end_matches(|c| c == ',' || c == '.' || c == '!').to_lowercase())
    // count the number of yes and no answers
    .fold((0,0), |(yes, no), a| match a.as_str() {
        "yes" => (yes + 1, no),
        "no" => (yes, no + 1),
        _x => (yes, no)
    })
}

fn build_client (at: &oauth2::AccessToken) -> Result<reqwest::blocking::Client, reqwest::Error> {
    let mut bearer_token = "Bearer ".to_string();
    bearer_token.push_str(&at.secret().to_string());
    let mut auth_header = reqwest::header::HeaderMap::new();
    auth_header.insert(AUTHORIZATION, HeaderValue::from_str(&bearer_token).unwrap());
    reqwest::blocking::ClientBuilder::new().default_headers(auth_header).user_agent("Run Counter").build()
}
