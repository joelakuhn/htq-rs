use std::{env, process::exit, fs::OpenOptions, vec};

use clap::{Arg, ArgAction, ArgMatches};
use scraper::{Html, Selector, ElementRef};
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
enum SelectorDirection {
    Document,
    Current,
}

struct SelectorSet {
    direction: SelectorDirection,
    selectors: Vec<Selector>,
}

struct MatchRecorder<'a> {
    count: usize,
    opts: &'a Opts<'a>,
    path: &'a String,
}

impl MatchRecorder<'_> {
    fn new<'a>(opts: &'a Opts<'a>, path: &'a String) -> MatchRecorder<'a> {
        return MatchRecorder {
            count: 0,
            opts: opts,
            path: path,
        }
    }

    #[allow(unused_must_use)]
    fn record<'a>(&mut self, el: &ElementRef, out: &'a mut dyn Write) {
        if self.opts.count {
            self.count += 1;
            return;
        }

        if self.opts.quiet {
            exit(0);
        }

        if self.opts.list {
            if self.opts.prefix { write!(out, "{}:", *self.path); }
            write!(out, "{}{}", *self.path, self.opts.nl);
        }

        if self.opts.attributes.len() > 0 {
            for attr in &self.opts.attributes {
                if self.opts.prefix { write!(out, "{}:", *self.path); }
                write!(out, "{}{}", el.value().attr(attr).unwrap_or(""), self.opts.nl);
            }
        }

        else if self.opts.text {
            if self.opts.prefix { write!(out, "{}:", *self.path); }
            for piece in el.text() {
                write!(out, "{}", piece);
            }
            write!(out, "{}", self.opts.nl);
        }

        else {
            if self.opts.prefix { write!(out, "{}:", *self.path); }
            write!(out, "{}{}", el.html(), self.opts.nl);
        }
    }

    #[allow(unused_must_use)]
    fn conclude<'a>(self, out: &'a mut dyn Write) {
        if self.opts.count {
            write!(out, "{}{}", self.count, self.opts.nl);
        }
    }
}

fn proces_fragment(document: &ElementRef, sets: &Vec<SelectorSet>, set_i: usize, opts: &Opts, out: &mut dyn Write, path: &String) {
    let direction = &sets.get(set_i).unwrap().direction;
    let mut recorder = MatchRecorder::new(opts, path);

    for selector in &sets.get(set_i).unwrap().selectors {
        let matches : Vec<ElementRef> = document.select(selector).collect();
if set_i + 1 < sets.len() {
            for el in matches {
                let next_set_i = set_i + 1;
                if matches!(direction, SelectorDirection::Current) {
                    proces_fragment(&el, sets, next_set_i, opts, out, path);
                }
                else {
                    proces_fragment(document, sets, next_set_i, opts, out, path);
                    break;
                }
            } }
        else {
            for el in matches {
                if matches!(direction, SelectorDirection::Current) {
                    recorder.record(&el, out);
                }
                else {
                    recorder.record(document, out);
                }
            }
        }
    }
    recorder.conclude(out);
}

fn process_path(path: &String, set: &Vec<SelectorSet>, set_i: usize, opts: &Opts, write: &mut dyn Write) {
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

    .arg(Arg::new("parent")
        .short('p')
        .long("parent")
        .help("Select the current element rather than the matched child")
        .action(ArgAction::SetTrue))

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

    let stdin_path = String::from("-");

    let args: Vec<String> = std::env::args().collect();
    let matches = parse_args(&args);
    let nl = if *matches.get_one::<bool>("print0").unwrap_or(&false) { "\0" } else { LINE_ENDING };
    let prefix = *matches.get_one::<bool>("prefix").unwrap_or(&false);
    let mut positional_args : Vec<&String> = matches.get_many::<String>("files").unwrap_or_default().collect();
    let out_opt = matches.get_one::<String>("output");
    let list = *matches.get_one::<bool>("list").unwrap_or(&false);
    let quiet = *matches.get_one::<bool>("quiet").unwrap_or(&false);
    let text = *matches.get_one::<bool>("text").unwrap_or(&false);
    let count = *matches.get_one::<bool>("count").unwrap_or(&false);
    let attributes : Vec<&String> = matches.get_many::<String>("attribute").unwrap_or_default().collect();

    if positional_args.len() == 0 {
        positional_args.push(&stdin_path);
    }
    else {
        println!("{:?}", positional_args);
    }

    let mut fileout;
    let mut stdout;
    let out: &mut dyn std::io::Write = match out_opt {
        Some(path) => {
            fileout = OpenOptions::new().write(true).open(path).expect("Could not open output file.");
            &mut fileout
        }
        None => {
            stdout = std::io::stdout();
            &mut stdout
        }
    };

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

    let mut selector_sets: Vec<SelectorSet> = vec![];
    let commands = args.split(|arg| arg == "|" || arg == "!");
    for (i, command) in commands.enumerate() {
        let mut vec_command: Vec<String> = Vec::from_iter(command.iter().map(|s| String::from(s)));
        if i > 0 {
            vec_command.insert(0, String::from(args.first().unwrap()));
        }

        let matches = parse_args(&vec_command.into());

        let direction = if *matches.get_one::<bool>("parent").unwrap_or(&false) {
            SelectorDirection::Document
        }
        else {
            SelectorDirection::Current
        };
        let selectors : Vec<Selector> = matches.get_many::<String>("selector").unwrap_or_default().map(|s| {
            Selector::parse(s).expect("Could not parse selector {}")
        }).collect();

        selector_sets.push(SelectorSet {
            direction: direction,
            selectors: selectors,
        });
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
