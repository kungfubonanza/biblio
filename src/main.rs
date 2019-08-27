extern crate config;

#[macro_use]
extern crate clap;

use reqwest;
use serde::Deserialize;

/// Data class that models the fields in the "passage_meta" field
/// contained in a JSON response from the ESV API.
///
/// Note that the variable names are identical to the field names in
/// the JSON.
#[derive(Deserialize, Debug)]
struct EsvApiResponsePassageMeta {
    canonical: String,
    chapter_start: Vec<u32>,
    chapter_end: Vec<u32>,
    prev_verse: Option<u32>, // is optional because there is no prev_verse for Genesis 1:1
    next_verse: Option<u32>, // is optional because there is no next_verse for Revelation 22:21
    prev_chapter: Option<Vec<u32>>, // is optional because there is no prev_chapter for Genesis 1:1
    next_chapter: Option<Vec<u32>>, // is optional because there is no next_chapter for Revelation 22:21
}

/// Data class that represents the JSON response from the ESV API.
///
/// Note that the variable names are identical to the field names in
/// the JSON.
#[derive(Deserialize, Debug)]
struct EsvApiResponse {
    query: String,
    canonical: String,
    parsed: Vec<Vec<u32>>,
    passage_meta: Vec<EsvApiResponsePassageMeta>,
    passages: Vec<String>,
}

/// Uses the command line arguments to build and return a tuple containing
/// all the API arguments and the verse(s) to be looked up.
fn get_api_request_arguments() -> (String, String) {
    // read the command line argument settings from the YAML file
    let yaml = load_yaml!("commandlineargs.yaml");

    // get all the matching arguments
    let matches = clap::App::from_yaml(yaml).get_matches();

    let mut options_string = String::new();

    let boolean_args: [&str; 12] = [
        ("include-passage-references"),
        ("include-verse-numbers"),
        ("include-first-verse-numbers"),
        ("include-footnotes"),
        ("include-footnotes-body"),
        ("include-headings"),
        ("include-short-copyright"),
        ("include-copyright"),
        ("include-passage-horizontal-lines"),
        ("include-heading-horizontal-lines"),
        ("include-selahs"),
        ("indent-poetry"),
    ];

    for arg in &boolean_args {
        // if the arg is present, then enable it
        if matches.is_present(arg) {
            options_string.push_str(&format!("&{}=true", arg).to_string());
        // if the arg is not present, then disable it
        } else {
            options_string.push_str(&format!("&{}=false", arg).to_string());
        }
    }

    // array of tuples, each tuple containing the name, minimum, and default
    // values for the argument
    let integer_args: [(&str, u32, u32); 6] = [
        ("horizontal-line-length", 1, 55),
        ("indent-paragraphs", 0, 2),
        ("include-poetry-lines", 0, 4),
        ("indent-declares", 0, 40),
        ("indent-psalm-doxology", 0, 30),
        ("line-length", 1, 0),
    ];

    for arg in &integer_args {
        // if the arg is present, then use the integer value the user
        // specified
        if matches.is_present(arg.0) {
            // TODO: handle out of range values
            let value = value_t!(matches.value_of(arg.0), u32).unwrap_or(arg.2);
            options_string.push_str(&format!("&{}={}", arg.0, value).to_string());
        } else {
        }
        // if the arg is not present, then use the API default by not passing
        // the argument as option
    }

    if matches.is_present("indent-using") {
        options_string.push_str(&format!("&indent-using={}", matches.value_of("indent-using").unwrap_or("space")).to_string());
    }

    // Calling .unwrap() is safe here because "VERSE" is required (if "VERSE"
    // wasn't required we could have used an 'if let' to conditionally get
    // the value)
    println!("Looking up this verse: {}", matches.value_of("VERSE").unwrap());

    let verse_string = matches.value_of("VERSE").unwrap().to_string();

    (verse_string, options_string)
}

/// Main function.
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

    let mut api_request_string = String::from("https://api.esv.org/v3/passage/text/?q=");

    let (verse_string, api_options) = get_api_request_arguments();

    api_request_string.push_str(&verse_string);
    api_request_string.push_str(&api_options);

    dbg!(api_request_string.as_str());

    let mut server_reply = client.get(api_request_string.as_str())
        .header(reqwest::header::AUTHORIZATION, esv_api_key)
        .send()?;

    match server_reply.status() {
        reqwest::StatusCode::OK => {
            let text = server_reply.text().unwrap();
            //let text = r#"{"query": "John 11:35", "canonical": "John 11:35", "parsed": [[43011035, 43011035]], "passage_meta": [{"canonical": "John 11:35", "chapter_start": [43011001, 43011057], "chapter_end": [43011001, 43011057], "prev_verse": 43011034, "next_verse": 43011036, "prev_chapter": [43010001, 43010042], "next_chapter": [43012001, 43012050]}], "passages": ["John 11:35\n\n  [35] Jesus wept. (ESV)"]}"#;

            let api_response: EsvApiResponse = serde_json::from_str(&text).unwrap();

            print!("{}", api_response.passages[0]);
        }
        s => {
            println!("Unable to retrieve {}. The server returned the following code: {:?}.", verse_string.to_string(), s);
            // TODO: return an error here
        }
    }

    Ok(())
}
