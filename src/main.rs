use indicatif::ProgressStyle;
use std::fs::{self};
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};

use clap::{App, Arg, ArgMatches};
use walkdir::WalkDir;

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
}

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

        Config {
            server,
            port,
            login,
            password,
            folder,
            directory,
            recursive,
            symlink,
        }
    }
}

fn main() {
    let matches = App::new("eml-replicator")
        .version("1.0")
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

    let tls = native_tls::TlsConnector::builder().build().unwrap();
    let client = imap::connect((conf.server.clone(), conf.port), conf.server, &tls).unwrap();
    let mut session = client.login(conf.login, conf.password).unwrap();
    let bar = indicatif::ProgressBar::new(emls_files.len() as u64);
    bar.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {msg} {pos:>7}/{len:7} {bar:40.cyan/blue}"),
    );
    bar.set_message("EML Copied");
    for eml in &emls_files {
        let rfc822 = fs::read_to_string(eml).expect("Failed to read eml file.");
        session
            .append(&conf.folder, &rfc822)
            .expect("Could not copy eml file to inbox.");
        bar.inc(1);
    }
    bar.finish();
}
