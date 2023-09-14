use std::{env, process::exit, fs::OpenOptions};

use clap::{Arg, ArgAction};
use scraper::{Html, Selector};

#[allow(unused_must_use)]
fn main() {
    #[cfg(windows)]
    const LINE_ENDING: &'static str = "\r\n";
    #[cfg(not(windows))]
    const LINE_ENDING: &'static str = "\n";

    let matches = clap::command!()
        .version("1.0.0")

        .arg(Arg::new("selector")
            .short('c')
            .long("css")
            .help("CSS selector")
            .action(ArgAction::Append))

        .arg(Arg::new("attribute")
            .short('a')
            .long("attr")
            .help("Extract an attribute value")
            .action(ArgAction::Append))

        .arg(Arg::new("text")
            .short('t')
            .long("text")
            .help("Print text content only")
            .action(ArgAction::SetTrue))

        .arg(Arg::new("prefix")
            .short('h')
            .long("prefix")
            .help("Print file name prefix")
            .action(ArgAction::SetTrue))

        .arg(Arg::new("list")
            .short('l')
            .long("list")
            .help("Only print matching file names")
            .action(ArgAction::SetTrue))

        .arg(Arg::new("count")
            .short('C')
            .long("count")
            .help("Print the number of matches")
            .action(ArgAction::SetTrue))

        .arg(Arg::new("quiet")
            .short('q')
            .long("quiet")
            .help("Suppress output")
            .action(ArgAction::SetTrue))

        .arg(Arg::new("print0")
            .short('0')
            .long("print0")
            .help("Null-terminate lines")
            .action(ArgAction::SetTrue))

        .arg(Arg::new("output")
            .short('o')
            .long("output")
            .help("Output file"))

        .arg(Arg::new("files")
            .action(ArgAction::Append))

        .get_matches();

    let mut selector_args : Vec<&String> = matches.get_many::<String>("selector").unwrap_or_default().collect();
    let mut positional_args : Vec<&String> = matches.get_many::<String>("files").unwrap_or_default().collect();
    let attributes : Vec<&String> = matches.get_many::<String>("attribute").unwrap_or_default().collect();
    let list = *matches.get_one::<bool>("list").unwrap_or(&false);
    let quiet = *matches.get_one::<bool>("quiet").unwrap_or(&false);
    let text = *matches.get_one::<bool>("text").unwrap_or(&false);
    let prefix = *matches.get_one::<bool>("prefix").unwrap_or(&false);
    let count = *matches.get_one::<bool>("count").unwrap_or(&false);
    let nl = if *matches.get_one::<bool>("print0").unwrap_or(&false) { "\0" } else { LINE_ENDING };
    let mut fileout;
    let mut stdout;
    let out : &mut dyn std::io::Write = match matches.get_one::<String>("output") {
        Some(path) => {
            fileout = OpenOptions::new().write(true).open(path).expect("Could not open output file.");
            &mut fileout
        }
        None => {
            stdout = std::io::stdout();
            &mut stdout
        }
    };

    if selector_args.len() == 0 && positional_args.len() > 0 {
        selector_args.push(positional_args.first().unwrap());
        positional_args.drain(0..1);
    }

    if selector_args.len() == 0 {
        eprintln!("You must specify at least one selector.");
        exit(1);
    }

    let selectors : Vec<Selector> = selector_args.iter().map(|s| {
        Selector::parse(s).expect("Could not parse selector {}")
    }).collect();

    let stdin_path = String::from("-");
    if positional_args.len() == 0 {
        positional_args.push(&stdin_path);
    }

    for path in positional_args {
        let file_result = if path == "-" {
            std::io::read_to_string(std::io::stdin())
        }
        else {
            std::fs::read_to_string(path)
        };

        match file_result {
            Ok(contents) => {
                let document = Html::parse_document(&contents);
                let mut match_count = 0;
                for sel in &selectors {
                    let matches : Vec<scraper::ElementRef<'_>> = document.select(&sel).collect();
                    if quiet && matches.len() > 0 {
                        exit(0);
                    }
                    if list && matches.len() > 0 {
                        if prefix { write!(out, "{}:", path); }
                        write!(out, "{}{}", path, nl);
                        break;
                    }
                    if count {
                        match_count += matches.len();
                        continue;
                    }
                    for el in matches {
                        if attributes.len() > 0 {
                            for attr in &attributes {
                                if prefix { write!(out, "{}:", path); }
                                write!(out, "{}{}", el.value().attr(attr).unwrap_or(""), nl);
                            }
                        }
                        else if text {
                            if prefix { write!(out, "{}:", path); }
                            for piece in el.text() {
                                write!(out, "{}", piece);
                            }
                            write!(out, "{}", nl);
                        }
                        else {
                            if prefix { write!(out, "{}:", path); }
                            write!(out, "{}{}", el.html(), nl);
                        }
                    }
                }
                if count {
                    write!(out, "{}{}", match_count, nl);
                }
            },
            _ => {}
        }
    }

    if quiet {
        exit(1);
    }
}
