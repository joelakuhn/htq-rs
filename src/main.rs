use std::{env, process::exit, fs::OpenOptions, vec};

use clap::{Arg, ArgAction, ArgMatches};
use scraper::{Html, Selector};
use std::io::Write;

struct Opts<'a> {
    attributes: Vec<&'a String>,
    list: bool,
    quiet: bool,
    text: bool,
    prefix: bool,
    count: bool,
    nl: &'a str,
}

#[allow(unused_must_use)]
fn proces_fragment(document: &scraper::ElementRef, set: &Vec<Vec<Selector>>, set_i: usize, opts: &Opts, out: &mut dyn Write, path: &String) {
    let mut match_count = 0;
    for sel in set.get(set_i).unwrap() {
        let matches : Vec<scraper::ElementRef> = document.select(&sel).collect();

        if set_i < set.len() - 1 {
            for el in matches {
                let next_set_i = set_i + 1;
                proces_fragment(&el, set, next_set_i, opts, out, path)
            }
            continue;
        }

        if opts.quiet && matches.len() > 0 {
            exit(0);
        }
        if opts.list && matches.len() > 0 {
            if opts.prefix { write!(out, "{}:", *path); }
            write!(out, "{}{}", *path, opts.nl);
            break;
        }
        if opts.count {
            match_count += matches.len();
            continue;
        }
        for el in matches {
            if opts.attributes.len() > 0 {
                for attr in &opts.attributes {
                    if opts.prefix { write!(out, "{}:", *path); }
                    write!(out, "{}{}", el.value().attr(attr).unwrap_or(""), opts.nl);
                }
            }
            else if opts.text {
                if opts.prefix { write!(out, "{}:", *path); }
                for piece in el.text() {
                    write!(out, "{}", piece);
                }
                write!(out, "{}", opts.nl);
            }
            else {
                if opts.prefix { write!(out, "{}:", *path); }
                write!(out, "{}{}", el.html(), opts.nl);
            }
        }
    }
    if opts.count {
        write!(out, "{}{}", match_count, opts.nl);
    }
}

fn process_path(path: &String, set: &Vec<Vec<Selector>>, set_i: usize, opts: &Opts, write: &mut dyn Write) {
    let file_result = if path == "-" {
        std::io::read_to_string(std::io::stdin())
    }
    else {
        std::fs::read_to_string(path)
    };

    match file_result {
        Ok(contents) => {
            let document = Html::parse_document(&contents);
            proces_fragment(&document.root_element(), set, set_i, opts, write, path);
        },
        _ => {}
    }
}

fn parse_args(args: &Vec<String>) -> ArgMatches {
    clap::command!()
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

    .get_matches_from(args)
}

fn main() {
    #[cfg(windows)]
    const LINE_ENDING: &'static str = "\r\n";
    #[cfg(not(windows))]
    const LINE_ENDING: &'static str = "\n";

    let args: Vec<String> = std::env::args().collect();
    let matches = parse_args(&args);

    let nl = if *matches.get_one::<bool>("print0").unwrap_or(&false) { "\0" } else { LINE_ENDING };
    let prefix = *matches.get_one::<bool>("prefix").unwrap_or(&false);
    let mut positional_args : Vec<&String> = matches.get_many::<String>("files").unwrap_or_default().collect();
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
    let stdin_path = String::from("-");
    if positional_args.len() == 0 {
        positional_args.push(&stdin_path);
    }
    let list = *matches.get_one::<bool>("list").unwrap_or(&false);
    let quiet = *matches.get_one::<bool>("quiet").unwrap_or(&false);
    let text = *matches.get_one::<bool>("text").unwrap_or(&false);
    let count = *matches.get_one::<bool>("count").unwrap_or(&false);
    let attributes : Vec<&String> = matches.get_many::<String>("attribute").unwrap_or_default().collect();

    // if selector_args.len() == 0 && positional_args.len() > 0 {
    //     selector_args.push(positional_args.first().unwrap());
    //     positional_args.drain(0..1);
    // }

    // if selector_args.len() == 0 {
    //     eprintln!("You must specify at least one selector.");
    //     exit(1);
    // }

    let mut opts = Opts{
        attributes: attributes,
        list: list,
        quiet: quiet,
        text: text,
        prefix: prefix,
        count: count,
        nl: nl,
    };

    let mut selector_sets: Vec<Vec<Selector>> = vec![];
    let commands = args.split(|arg| arg == "|" || arg == "!");
    let mut i = 0;
    for command in commands {
        let mut vec_command: Vec<String> = Vec::from_iter(command.iter().map(|s| String::from(s)));
        if i > 0 {
            vec_command.insert(0, String::from(args.first().unwrap()));
        }
        i += 1;

        let matches = parse_args(&vec_command.into());

        let selectors : Vec<Selector> = matches.get_many::<String>("selector").unwrap_or_default().map(|s| {
            Selector::parse(s).expect("Could not parse selector {}")
        }).collect();

        selector_sets.push(selectors);
    }

    if selector_sets.len() > 0 {
        for path in positional_args {
            process_path(path, &selector_sets, 0, &mut opts, out);
        }
    }
    else {
        eprintln!("You must have at least one selector.");
        exit(1);
    }

    if opts.quiet {
        exit(1);
    }
}
