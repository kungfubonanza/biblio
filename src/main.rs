extern crate config;

use reqwest;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
/// Data class that models the fields in the "passage_meta" field
/// contained in a JSON response from the ESV API.
///
/// Note that the variable names are identical to the field names in
/// the JSON.
struct EsvApiResponsePassageMeta {
    canonical: String,
    chapter_start: Vec<u32>,
    chapter_end: Vec<u32>,
    prev_verse: Option<u32>, // is optional because there is no prev_verse for Genesis 1:1
    next_verse: Option<u32>, // is optional because there is no next_verse for Revelation 22:21
    prev_chapter: Option<Vec<u32>>, // is optional because there is no prev_chapter for Genesis 1:1
    next_chapter: Option<Vec<u32>>, // is optional because there is no next_chapter for Revelation 22:21
}

#[derive(Deserialize, Debug)]
/// Data class that represents the JSON response from the ESV API.
///
/// Note that the variable names are identical to the field names in
/// the JSON.
struct EsvApiResponse {
    query: String,
    canonical: String,
    parsed: Vec<Vec<u32>>,
    passage_meta: Vec<EsvApiResponsePassageMeta>,
    passages: Vec<String>,
}

fn main() -> Result<(), reqwest::Error> {

    // grab the API key from Settings.toml or the ESV_API_KEY
    // environmental variable
    let mut settings = config::Config::default();
    settings
        .merge(config::File::with_name("Settings")).unwrap()
        .merge(config::Environment::with_prefix("ESV")).unwrap();

    // TODO: handle a failure to get an api key
    let esv_api_key = settings.get_str("API_KEY").unwrap();

    let client = reqwest::Client::new();

    let mut req_string = String::from("https://api.esv.org/v3/passage/text/?q=");

    let ref_string = String::from("John+3:16");

    req_string.push_str(&ref_string);

    let mut server_reply = client.get(req_string.as_str())
        .header(reqwest::header::AUTHORIZATION, esv_api_key)
        .send()?;

    match server_reply.status() {
        reqwest::StatusCode::OK => {
            let text = server_reply.text().unwrap();
            //let text = r#"{"query": "John 11:35", "canonical": "John 11:35", "parsed": [[43011035, 43011035]], "passage_meta": [{"canonical": "John 11:35", "chapter_start": [43011001, 43011057], "chapter_end": [43011001, 43011057], "prev_verse": 43011034, "next_verse": 43011036, "prev_chapter": [43010001, 43010042], "next_chapter": [43012001, 43012050]}], "passages": ["John 11:35\n\n  [35] Jesus wept. (ESV)"]}"#;

            let api_response: EsvApiResponse = serde_json::from_str(&text).unwrap();

            println!("{}", api_response.passages[0]);
        }
        s => {
            println!("Unable to retrieve {}. The server returned the following code: {:?}.", ref_string, s);
            // TODO: return an error here
        }
    }

    Ok(())
}
