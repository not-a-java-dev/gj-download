use std::{collections::HashMap, fs};
use std::io::prelude::*;
use reqwest::blocking::Client;
use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use clap::Parser;
use flate2::read::GzDecoder;
use colored::*;
use inflate::inflate_bytes_zlib;

/// Simple Geometry Jump Level Downloader
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// The ID of the level to download, to download daily levels use "daily" and "weekly" for
    /// weekly ones
    #[arg(short, long)]
    level: String,
    /// Whether to decrypt or not [DEFAULT = TRUE]
    #[arg(short, long, default_value_t = false)]
    dont_decrypt: bool,
}
fn request_gj(api: &str, form: HashMap<String, String>) -> String {
    let client = Client::new();
    let res = client.post("http://www.boomlings.com/database/".to_owned() + api + ".php")
        .header("User-Agent", "")
        .form(&form)
        .send()
        .expect("idk")
        .text()
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

fn main() {
    let args = Cli::parse();
    let mut form = HashMap::new();
    let mut lvl_id = args.level;
    if lvl_id.clone() == "daily".to_string() {
        lvl_id = "-1".to_string();
    } else if lvl_id.clone() == "weekly".to_string() {
        lvl_id = "-2".to_string();
    }
    form.insert("secret".to_string(), "Wmfd2893gb7".to_string());
    form.insert("levelID".to_string(), lvl_id);
    println!("{}", "[ DOWNLOADING ]".blue().bold());
    let response = request_gj("downloadGJLevel22", form);
    if response == "-1" {             
        println!("{} Level ID does not exist or it's unlisted!", "[ ERROR ]".red().bold());
        return;
    }
    let parsed = parse_universal(&response, ":");
    println!("{}", "[ ENDED DOWNLOADING ]".blue().bold());
    let level = parsed.get(&"4"); // 4th key is the level data, a big blob of base64 that was gzipped
    match parsed.get(&"2") { // Level Name
        Some(x) => {
            println!("Level \"{}\" downloaded", x);
        }
        None => {
            println!("Level downloaded");
        }
    }
    match parsed.get(&"3") { // Description base64 encoded
        Some(x) => {
            print!("Description -> ");
            let description = URL_SAFE.decode(x.as_bytes()).unwrap();
            let mut str_description = String::new();
            for b in description {
                str_description.push(char::from(b));
            }
            println!("{}", str_description);
        }
        None => {
            println!("Description -> <No description provided>");
        }
    }
    match parsed.get(&"5") {
        Some(x) => {
            print!("Version: {} | ", x);
        }
        None => {
            print!("Version: UNKNOWN | ");
        }
    }
    match parsed.get(&"10") {
        Some(x) => {
            print!("Downloads: {} | ", x);
        }
        None => {
            print!("Downloads: UNKNOWN | ");
        }
    }
    match parsed.get(&"28") {
        Some(x) => {
            print!("Uploaded {} ago | ", x);
        }
        None => {
            print!("Uploaded UNKNOWN ago | ");
        }
    }
    match parsed.get(&"29") {
        Some(x) => {
            print!("Last Updated {} ago ", x);
        }
        None => {
            print!("Last Updated UNKNOWN ago ");
        }
    }
    println!();
    // match parsed.get(&"13") { // Version (this was used before pre1.9 levels could be decrypted, now they can be)
    //     Some(x) => {
    //         let x_parsed = x.to_string().parse::<i32>().unwrap();
    //         if x_parsed < 19 {
    //             println!("{} Level version is less than 19! Cannot decrypt level", "[ WARNING ]".yellow().bold());
    //             forced_decrypt = true;
    //             if args.force_decrypt {
    //                 println!("{} Force decrypt argument was passed! Attempting to decrypt level...", "[ INFO ]".green().bold());
    //                 forced_decrypt = false;
    //             }
    //         }
    //     }
    //     None => {
    //         println!("{} Level version was not found, decryption might not work", "[ INFO ]".green().bold());
    //     }
    // }
    match level {
        Some(x) => {
            let bytes = x.as_bytes();
            if args.dont_decrypt != false {
                println!("Not decrypting! Saving now...");
                fs::write("level.txt", x).unwrap();
                return;
            }
            println!("{}", "[ DECRYPTING ]".green().bold());
            let data = URL_SAFE.decode(bytes).unwrap();

            let mut gunzipped_data = GzDecoder::new(data.as_slice());
            let mut data_string = String::new();
            let dat = gunzipped_data.read_to_string(&mut data_string);
            let mut pre = false;
            if dat.is_ok() {
                dat.unwrap();
            } else { // probably a pre 1.9 level or some corrupt thing idk im dumb
                pre = true;
                let decoded = inflate_bytes_zlib(data.as_slice());
                fs::write("level.txt", decoded.unwrap()).unwrap(); // this looks like bird poop
            }
            println!("{}", "[ ENDED DECRYPTING ]".green().bold());
            println!("Saving now...");
            if pre {
                return;
            }
            fs::write("level.txt", data_string).unwrap();
        }
        None => {
            panic!("Level isn't here!! What Happened? {:?}", parsed);
        }
    }
}
