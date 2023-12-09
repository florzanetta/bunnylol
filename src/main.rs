#[macro_use]
extern crate rocket;

use rocket::response::content::RawHtml;
use rocket::response::Redirect;

use rocket::form::Form;
use rocket::State;
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::collections::HashMap;
use std::fs;
use std::sync::Mutex;

extern crate percent_encoding;

use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};

const FRAGMENT: &AsciiSet = &CONTROLS.add(b' ').add(b'"').add(b'<').add(b'>').add(b'`');

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
struct Config {
    config_file: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct SiteList(Mutex<HashMap<String, HashMap<String, String>>>);

#[derive(FromForm)]
struct NewSiteData {
    key: String,
    name: String,
    url: String,
}

fn load_config(file_path: &String) -> Result<SiteList, Box<dyn std::error::Error>> {
    // Read the file to a string
    let file_content = fs::read_to_string(file_path)?;
    // Deserialize the string into Data
    let deserialized_data: SiteList = serde_yaml::from_str(&file_content)?;
    Ok(deserialized_data)
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

// TODO: merge add-site form with list GET
#[get("/add-site")]
fn add_site_form() -> RawHtml<&'static str> {
    RawHtml(
        r#"
        <html>
            <head>
                <meta name="color-scheme" content="light dark">
            </head>
            <body>
                <form action="/add-site" method="post">
                    <label for="key">Key:</label>
                    <input type="text" id="key" name="key"><br><br>
                    <label for="name">Name:</label>
                    <input type="text" id="name" name="name"><br><br>
                    <label for="url">URL:</label>
                    <input type="text" id="url" name="url"><br><br>
                    <input type="submit" value="Add Site">
                </form>
            </body>
        </html>
    "#,
    )
}

#[post("/add-site", data = "<new_site_data>")]
async fn add_site(
    site_list: &State<SiteList>,
    config: &State<String>,
    new_site_data: Form<NewSiteData>,
) -> String {
    let mut sites = site_list.0.lock().unwrap();

    // Create the inner HashMap for the new site
    let mut site_info = HashMap::new();
    site_info.insert("name".to_string(), new_site_data.name.clone());
    site_info.insert("url".to_string(), new_site_data.url.clone());

    // Insert the new site into the shared HashMap
    sites.insert(new_site_data.key.clone(), site_info);

    let serialized_data = serde_yaml::to_string(&*sites).unwrap();
    let _ = fs::write(config.inner(), serialized_data);
    "Added/Updated site".to_string()
}

// TODO: add remove form
// TODO: move html to template
// TODO: add deploy script and systemd configs
#[get("/list")]
fn list(site_list: &State<SiteList>) -> String {
    let sites = site_list.0.lock().unwrap();

    let mut result = String::new();

    for (key, site) in sites.iter() {
        result.push_str(&format!("Key: {}\n", key));
        for (site_key, site_value) in site.iter() {
            result.push_str(&format!("  {}: {}\n", site_key, site_value));
        }
    }

    result
}

fn split_first_word(s: &str) -> (&str, &str) {
    if let Some(index) = s.find(char::is_whitespace) {
        let (first, rest) = s.split_at(index);
        (first, rest.trim_start())
    } else {
        (s, "")
    }
}

#[get("/search?<cmd>")]
fn search(site_list: &State<SiteList>, cmd: String) -> Redirect {
    let (first_word, the_rest) = split_first_word(&cmd);
    let encoded_query = utf8_percent_encode(the_rest, FRAGMENT).to_string();

    let redirect_url: String;
    let sites = site_list.0.lock().unwrap();
    if sites.contains_key(first_word) {
        let url = sites.get(first_word).unwrap().get("url").unwrap().clone();
        if url.contains("{}") {
            redirect_url = url.replace("{}", &encoded_query);
        } else {
            redirect_url = url;
        }
    } else {
        let encoded_query = utf8_percent_encode(&cmd, FRAGMENT).to_string();
        redirect_url = format!("https://google.com/search?q={}", encoded_query);
    }

    Redirect::to(redirect_url)
}

#[launch]
fn rocket() -> _ {
    let rocket = rocket::build();
    let figment = rocket.figment();

    let config_file: String = figment
        .extract_inner("config_file")
        .expect("Failed to extract 'config_file'");

    let list: SiteList = load_config(&config_file).unwrap();

    rocket
        .manage(list)
        .manage(config_file)
        .mount("/", routes![index, search, list, add_site, add_site_form])
}
