use std::{collections::HashMap, fs};
use std::io::prelude::*;
use reqwest::Client;
use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use clap::Parser;
use flate2::read::GzDecoder;

/// Simple Geometry Jump Level Downloader
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// The ID of the level to download
    #[arg(short, long)]
    level: String,
    /// Whether to decrypt or not [DEFAULT = TRUE]
    #[arg(short, long, default_value_t = false)]
    dont_decrypt: bool,
}
async fn request_gj(api: &str, form: HashMap<String, String>) -> String {
    let client = Client::new();
    let res = client.post("http://www.boomlings.com/database/".to_owned() + api + ".php")
        .header("User-Agent", "")
        .form(&form)
        .send()
        .await
        .expect("idk")
        .text()
        .await
        .expect("idk but something about payload");
    res
}

fn parse_universal<'a>(string: &'a str, sep: &'a str) -> HashMap<&'a str, &'a str> {
    let mut obj_new = HashMap::new();
    let obj = string.split(sep);
    let obj_vec: Vec<&str> = obj.clone().collect();
    for val in obj.clone().enumerate() {
        if val.0 % 2 != 0 {
            obj_new.insert(obj_vec[val.0 - 1], val.1);
        }
    }
    obj_new
}



#[tokio::main]
async fn main() {
    let args = Cli::parse();
    let mut form = HashMap::new();
    form.insert("secret".to_string(), "Wmfd2893gb7".to_string());
    form.insert("levelID".to_string(), args.level);
    println!("[ DOWNLOADING ]");
    let response = request_gj("downloadGJLevel22", form).await;
    let parsed = parse_universal(&response, ":");
    println!("[ ENDED DOWNLOADING ]");
    let level = parsed.get(&"4"); // 4th key is the level data, a big blob of base64 that was gzipped
    match parsed.get(&"2") {
        Some(x) => {
            println!("Level \"{}\" downloaded", x);
        }
        None    => {
            println!("Level downloaded");
        }
    }
    match parsed.get(&"3") {
        Some(x) => {
            print!("-> ");
            let description = URL_SAFE.decode(x.as_bytes()).unwrap();
            let mut str_description = String::new();
            for b in description {
                str_description.push(char::from(b));
            }
            println!("{}", str_description);
        }
        None    => {
            println!("-> ");
        }
    }
    match level {
        Some(x) => {
            let bytes = x.as_bytes();
            if args.dont_decrypt != false {
                println!("Not decrypting! Saving now...");
                fs::write("level.txt", x).unwrap();
                return;
            }
            println!("[ DECRYPTING ]");
            let data = URL_SAFE.decode(bytes).unwrap();

            let mut d = GzDecoder::new(data.as_slice());
            let mut s = String::new();
            d.read_to_string(&mut s).unwrap();
            println!("[ ENDED DECRYPTING ]");
            println!("Saving now...");
            fs::write("level.txt", s).unwrap();
        }
        None           => {
            panic!("Level isn't here!! What Happened? {:?}", parsed);
        }
    }
}