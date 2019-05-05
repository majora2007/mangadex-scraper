extern crate reqwest;
extern crate regex;
extern crate clap;
extern crate zip;

use clap::{Arg, App};
use reqwest::Url;
use std::fs::File;
use std::io;
use std::io::{Read, Write, ErrorKind};


fn main() -> Result<(), reqwest::UrlError> {
    // command line arguments
    let args = App::new("mangadex-scraper")
        .version("0.2.1")
        .author("dyedquartz <dyedquartz@gmail.com>")
        .about("Scapes manga off of mangadex.org")
        .arg(Arg::with_name("id")
             .help("ID of the directory to download")
             .required(true)
             .index(1))
        .arg(Arg::with_name("compress")
             .short("c")
             .long("compress")
             .value_name("ARCHIVE_OUTPUT")
             .help("Compresses into a .cbz")
             .takes_value(true))
        .arg(Arg::with_name("remove")
             .long("remove")
             .help("Remove file after downloading. Most useful for cleanup after compressing"))
        .get_matches();


    let base_url = Url::parse("https://s2.mangadex.org/data/")?;
    let prefixes = vec!["", "x", "s", "K", "V", "v", "z", "q", "r", "k", "D", "a", "G", "m", "T", "R", "n", "w", "U", "S"];
    let id: &str = &args.value_of("id").unwrap();
    let mut pre = String::new();

    let client = reqwest::Client::new();
    
    // testing id directory
    let url = base_url.join(id)?;
    let resp = client.get(url).send().unwrap();
    match resp.status() {
        reqwest::StatusCode::FORBIDDEN => println!("Correct ID Path"),
        reqwest::StatusCode::NOT_FOUND => panic!("Incorrect ID Path"),
        _ => panic!("Unknown ID Path"),
    }

    
    // grabbing correct file prefix
    for prefix in prefixes {
        let url = base_url.join(&format!("{}/{}1.png",id, prefix))?;
        println!("{:?}", url);
        let resp = client.get(url).send().unwrap();
        if resp.status() == reqwest::StatusCode::OK {
            pre = String::from(prefix);
            break;
        }
        println!("{:?}",resp.status());
    }

    println!("File Prefix: {}", pre);

   
    // downloading files
    let mut i = 1;
    loop {
		let re = regex::Regex::new(r"\b\d\b").unwrap();
        let f = &*i.to_string();
        let f = re.replace_all(f, "0$0");


        //fs::create_dir_all(format!("{}", args[3])).unwrap();
        let url = base_url.join(&format!("{}/{}{}.png",id, pre, i))?;

        let mut resp = client.get(url).send().unwrap();

        if resp.status() == reqwest::StatusCode::OK {
            let mut out = File::create(format!("{}.png", f)).expect("failed to create file");
            io::copy(&mut resp, &mut out).expect("failed to copy");
        } else {
            let url = base_url.join(&format!("{}/{}{}.jpg",id, pre, i))?;
    
            let mut resp = client.get(url).send().unwrap();

            if resp.status() == reqwest::StatusCode::OK {
                let mut out = File::create(format!("{}.png", f)).expect("failed to create file");
                io::copy(&mut resp, &mut out).expect("failed to copy");
            } else {
                println!("{:?} no more files to download", resp.status());
                break;
            }
        }
        println!("Downloaded {}", f);
        i += 1;
    }

    if args.is_present("compress") {
        // create archive + buffer
        let mut buffer = Vec::new();
        let options = zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Bzip2);
        let mut archive = File::create(format!("{}.cbz", args.value_of("compress").unwrap())).unwrap();
        let mut writer = zip::write::ZipWriter::new(&mut archive);
        println!("created writer and archive");

        for archive_file in 1..i {
		    let re = regex::Regex::new(r"\b\d\b").unwrap();
            let f = &*archive_file.to_string();
            let f = re.replace_all(f, "0$0");
            let mut path = format!("{}.png", f);
            
            let image = File::open(&path);
            let mut image = match image {
                Ok(file) => file,
                Err(error) => match error.kind() {
                    ErrorKind::NotFound => match File::open(format!("{}.jpg", f)) {
                        Ok(jpg) => {
                            path = format!("{}.png", f);
                            jpg
                        }
                        Err(e) => panic!("problem opening file for archiving {:?}", e),
                    },
                    other_error => panic!("problem opening file for archiving {:?}", other_error),
                },
            };
            

            image.read_to_end(&mut buffer).unwrap();

            writer.start_file(format!("{}.png", f), options).unwrap();
            writer.write_all(&*buffer).unwrap();
            buffer.clear();
            println!("Compressed {}", path);
            if args.is_present("remove") {
                std::fs::remove_file(&path).unwrap();
                println!("Removed {}", path);
            }
        }
        writer.finish().unwrap();
    }
    Ok(())
}
