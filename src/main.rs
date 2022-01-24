#[macro_use]
extern crate lazy_static;

use indicatif::ProgressStyle;
use regex::bytes::Regex;
use std::fs;
use std::io::{Error, ErrorKind};
use std::net::TcpStream;
use std::path::{Path, PathBuf};

use clap::{App, Arg, ArgMatches};
use imap::Session;
use native_tls::{TlsConnector, TlsStream};
use rand::distributions::Alphanumeric;
use rand::Rng;
use walkdir::WalkDir;

fn push_ext(list: &mut Vec<PathBuf>, entry: &Path, ext_used: &str) {
    let path = entry;
    if path.is_file() {
        let ext = path.extension().map(|x| x.to_str().unwrap()).unwrap_or("");
        if ext_used == ext {
            list.push(path.to_path_buf());
        }
    }
}

fn list_eml_file(
    dir: &Path,
    recursive: bool,
    follow_symlink: bool,
) -> std::io::Result<Vec<PathBuf>> {
    if dir.is_dir() {
        let mut emls: Vec<PathBuf> = Vec::new();
        if recursive {
            for entry in WalkDir::new(dir).follow_links(follow_symlink) {
                let entry = entry?;
                push_ext(&mut emls, entry.path(), "eml");
            }
        } else {
            for entry in fs::read_dir(dir)? {
                let entry = entry?.path();
                push_ext(&mut emls, &entry, "eml");
            }
        }
        return Ok(emls);
    }
    return Err(Error::new(
        ErrorKind::InvalidInput,
        "the path was not a directory.",
    ));
}

fn randomize_message_id(eml: &[u8]) -> Result<Vec<u8>, String> {
    lazy_static! {
        static ref MID_RE: Regex = Regex::new(r"(?imu)^message-id:.+$").unwrap();
    }

    let mut new_eml = Vec::new();

    let header_pos = MID_RE.find(eml);
    if header_pos.is_none() {
        return Err("Could not find Message-ID in the EML.".to_string());
    }

    let (fpart, lpart) = eml.split_at(header_pos.unwrap().start());
    new_eml.extend_from_slice(fpart);

    let lpart_str = std::str::from_utf8(lpart)
        .map_err(|e| format!("UTF error: {e:?}"))?;
    let (_mid, rest) = lpart.split_at(lpart_str.find('\n').expect("Malformed Message-ID."));
    new_eml.extend_from_slice("Message-ID: ".as_bytes());
    let rand_string: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect();
    new_eml.extend_from_slice(rand_string.as_bytes());
    new_eml.extend_from_slice(rest);
    return Ok(new_eml);
}

#[derive(Clone, Debug)]
struct Config {
    server: String,
    port: u16,
    login: String,
    password: String,
    folder: String,
    directory: String,
    recursive: bool,
    symlink: bool,
    random_id: bool,
    skip_verify_cert: bool,
}

impl Config {
    pub fn new(matches: ArgMatches) -> Config {
        let server = String::from(matches.value_of("server").unwrap());
        let port = matches
            .value_of("server_port")
            .unwrap()
            .parse::<u16>()
            .unwrap();
        let login = String::from(matches.value_of("login").unwrap());
        let password = String::from(matches.value_of("password").unwrap());
        let folder = String::from(matches.value_of("folder").unwrap());
        let directory = String::from(matches.value_of("DIR").unwrap());

        let recursive = matches.is_present("recursive");
        let symlink = matches.is_present("symlink");
        let random_id = matches.is_present("random-message-id");
        let skip_verify_cert = matches.is_present("skip-verify-cert");

        Config {
            server,
            port,
            login,
            password,
            folder,
            directory,
            recursive,
            symlink,
            random_id,
            skip_verify_cert
        }
    }
}

fn main() {
    let matches = App::new("eml-replicator")
        .version("1.1")
        .author("Maël Naccache Tüfekçi")
        .about("A tool that read EML files and copy them to a IMAP mailbox.")
        .arg(
            Arg::with_name("server")
                .value_name("IMAP_SERVER")
                .help("IMAP server to connect to.")
                .takes_value(true)
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("server_port")
                .long("port")
                .value_name("IMAP_SERVER_PORT")
                .help("Port to connect to the imap server.")
                .takes_value(true)
                .default_value("993"),
        )
        .arg(
            Arg::with_name("login")
                .short("l")
                .long("login")
                .value_name("LOGIN")
                .help("login of the mailbox.")
                .takes_value(true)
                .default_value(""),
        )
        .arg(
            Arg::with_name("password")
                .short("p")
                .long("password")
                .value_name("PASSWORD")
                .help("password of the mailbox.")
                .takes_value(true)
                .default_value(""),
        )
        .arg(
            Arg::with_name("folder")
                .short("f")
                .long("folder")
                .value_name("FOLDER")
                .help("IMAP Folder in which to put the EMLs.")
                .takes_value(true)
                .default_value("INBOX"),
        )
        .arg(
            Arg::with_name("recursive")
                .short("r")
                .long("recursive")
                .help("Goes through the directory recursively to find EML files."),
        )
        .arg(
            Arg::with_name("symlink")
                .short("s")
                .long("follow-symlink")
                .help("Follow symlink when crawling the directory recursively."),
        )
        .arg(
            Arg::with_name("random-message-id")
                .long("random-message-id")
                .help("Randomize the Message-ID in the emls before sending them."),
        )
        .arg(
            Arg::with_name("skip-verify-cert")
                .long("skip-verify-cert")
                .help("Skip checking server certificate when connecting over TLS."),
        )
        .arg(
            Arg::with_name("DIR")
                .help("Directory in which to get the EML files.")
                .required(true)
                .index(2)
                .default_value("."),
        )
        .get_matches();

    let conf = Config::new(matches);
    let emls_files =
        list_eml_file(Path::new(&conf.directory), conf.recursive, conf.symlink).unwrap();

    println!("EML found:");
    for path in &emls_files {
        println!("- {}", path.to_str().unwrap_or(""));
    }

    if conf.random_id {
        println!("Randomizing Message-IDs.")
    }

    let mut session = connect(&conf);
    let bar = indicatif::ProgressBar::new(emls_files.len() as u64);
    bar.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {msg} {pos:>7}/{len:7} {bar:40.cyan/blue}"),
    );
    bar.set_message("EML Copied");
    for eml in &emls_files {
        let mut rfc822 = fs::read(eml).expect("Failed to read eml file.");
        if conf.random_id {
            let randomize_id = randomize_message_id(&rfc822);
            if randomize_id.is_err() {
                println!(
                    "Could not find Message-ID for file {}, skipping.",
                    eml.to_string_lossy()
                );
            } else {
                rfc822 = randomize_id.unwrap();
            }
        }

        let send_res = session.append(&conf.folder, &rfc822)
            .finish();
        if send_res.is_err() {
            // we might have been disconnected so we retry.
            let _ = session.close();
            session = connect(&conf);
            session
                .append(&conf.folder, &rfc822)
                .finish()
                .expect("Could not copy email.");
        }

        bar.inc(1);
    }
    bar.finish();
}

fn connect(conf: &Config) -> Session<TlsStream<TcpStream>> {
    let skip_verify_cert = conf.skip_verify_cert;
    let client = imap::ClientBuilder::new(&conf.server, conf.port)
        .connect(|domain, tcp| {
            let ssl_conn = TlsConnector::builder()
                .danger_accept_invalid_certs(skip_verify_cert)
                .build()?;
            Ok(TlsConnector::connect(&ssl_conn, domain, tcp)?)
        }).unwrap();
    client.login(&conf.login, &conf.password).unwrap()
}
