use std::{env, fs::OpenOptions, io::IsTerminal, process::exit, vec};

use clap::{Arg, ArgAction, ArgMatches};
use scraper::{Html, Selector, ElementRef};
use std::io::Write;
use regex::Regex;

struct Opts<'a> {
    attributes: Vec<&'a String>,
    list: bool,
    quiet: bool,
    text: bool,
    prefix: bool,
    trim: bool,
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
    search: Vec<String>,
    regex: Vec<Regex>,
}

#[allow(unused_must_use)]
fn highlight_prefix(out: &mut dyn Write, str: &str) {
    const COLORS_MAGENTA: &str = "\x1b[35m";
    const COLORS_RESET: &str = "\x1b[0m";

    if std::io::stdout().is_terminal() {
        write!(out, "{}{}:{}", COLORS_MAGENTA, str, COLORS_RESET);
    }
    else {
        write!(out, "{}:", str);
    }
}

struct MatchRecorder<'a> {
    count: usize,
    opts: &'a Opts<'a>,
    path: &'a String,
    out: &'a mut dyn Write,
}

impl MatchRecorder<'_> {
    fn new<'a>(opts: &'a Opts<'a>, path: &'a String, out: &'a mut dyn Write) -> MatchRecorder<'a> {
        return MatchRecorder {
            count: 0,
            opts: opts,
            path: path,
            out: out,
        }
    }

    #[allow(unused_must_use)]
    fn record<'a>(&mut self, el: &ElementRef) {
        if self.opts.count {
            self.count += 1;
            return;
        }

        if self.opts.quiet {
            exit(0);
        }

        if self.opts.list {
            if self.opts.prefix { highlight_prefix(self.out, self.path); }
            write!(self.out, "{}{}", *self.path, self.opts.nl);
        }

        if self.opts.attributes.len() > 0 {
            for attr in &self.opts.attributes {
                if self.opts.prefix { highlight_prefix(self.out, self.path); }
                if self.opts.trim {
                    write!(self.out, "{}{}", el.value().attr(attr).unwrap_or("").trim(), self.opts.nl);
                }
                else {
                    write!(self.out, "{}{}", el.value().attr(attr).unwrap_or(""), self.opts.nl);
                }
            }
        }

        else if self.opts.text {
            if self.opts.prefix { highlight_prefix(self.out, self.path); }
            for piece in el.text() {
                if self.opts.trim {
                    write!(self.out, "{}", piece.trim());
                }
                else {
                    write!(self.out, "{}", piece);
                }
            }
            write!(self.out, "{}", self.opts.nl);
        }

        else {
            if self.opts.prefix { highlight_prefix(self.out, self.path); }
            if self.opts.trim {
                write!(self.out, "{}{}", el.html().trim(), self.opts.nl);
            }
            else {
                write!(self.out, "{}{}", el.html(), self.opts.nl);
            }
        }
    }

    #[allow(unused_must_use)]
    fn conclude<'a>(self) {
        if self.opts.count && self.count > 0 {
            if self.opts.prefix { highlight_prefix(self.out, self.path); }
            write!(self.out, "{}{}", self.count, self.opts.nl);
        }
    }
}

fn fragment_matches_search(fragment: &ElementRef, set: &SelectorSet) -> bool {
    let is_match = (set.search.is_empty() && !set.regex.is_empty()) || {
        let fragment_text = String::from_iter(fragment.text());
        (!set.regex.is_empty() && set.regex.iter().any(|re| re.is_match(&fragment_text)))
        || (!set.search.is_empty() && set.search.iter().any(|search_str| fragment_text.contains(search_str)))
    };
    return is_match;
}

fn proces_fragment(document: &ElementRef, sets: &Vec<SelectorSet>, set_i: usize, opts: &Opts, recorder: &mut MatchRecorder) {
    if set_i >= sets.len() {
        recorder.record(&document);
        return;
    }

    let set = sets.get(set_i).unwrap();
    let direction = &set.direction;
    let selectors = &set.selectors;

    if selectors.is_empty() {
        if fragment_matches_search(document, set) {
            proces_fragment(document, sets, set_i + 1, opts, recorder);
        }
    }

    for selector in selectors {
        for el in document.select(selector) {
            let next_set_i = set_i + 1;
            if matches!(direction, SelectorDirection::Current) {
                proces_fragment(&el, sets, next_set_i, opts, recorder);
            }
            else {
                proces_fragment(document, sets, next_set_i, opts, recorder);
            }
        }
    }
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
            let mut recorder = MatchRecorder::new(opts, path, write);
            proces_fragment(&document.root_element(), set, set_i, opts, &mut recorder);
            recorder.conclude();
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
        .help_heading("Selectors")
        .action(ArgAction::Append))

    .arg(Arg::new("search")
        .short('s')
        .long("search")
        .action(ArgAction::Append)
        .help("Search String (matches against all elements)")
        .help_heading("Selectors")
        .group("stream-relative"))

    .arg(Arg::new("regex")
        .short('r')
        .long("regex")
        .action(ArgAction::Append)
        .help("Search regex")
        .help_heading("Selectors")
        .group("stream-relative"))

    .arg(Arg::new("parent")
        .short('p')
        .long("parent")
        .help("Select the current element rather than the matched child")
        .help_heading("Selectors")
        .action(ArgAction::SetTrue))

    .arg(Arg::new("attribute")
        .short('a')
        .long("attr")
        .help("Extract an attribute value")
        .help_heading("Output")
        .action(ArgAction::Append))

    .arg(Arg::new("text")
        .short('t')
        .long("text")
        .help("Print text content only")
        .help_heading("Output")
        .action(ArgAction::SetTrue))

    .arg(Arg::new("prefix")
        .short('h')
        .long("prefix")
        .help("Print file name prefix")
        .help_heading("Output")
        .action(ArgAction::SetTrue))

    .arg(Arg::new("no-prefix")
        .short('H')
        .long("no-prefix")
        .help("Suppress file name prefix")
        .help_heading("Output")
        .action(ArgAction::SetTrue))

    .arg(Arg::new("list")
        .short('l')
        .long("list")
        .help("Only print matching file names")
        .help_heading("Output")
        .action(ArgAction::SetTrue))

    .arg(Arg::new("trim")
        .short('T')
        .long("trim")
        .help("Trim leading and trailing whitespace")
        .help_heading("Output")
        .action(ArgAction::SetTrue))

    .arg(Arg::new("count")
        .short('C')
        .long("count")
        .help("Print the number of matches")
        .help_heading("Output")
        .action(ArgAction::SetTrue))

    .arg(Arg::new("quiet")
        .short('q')
        .long("quiet")
        .help("Suppress output")
        .help_heading("Output")
        .action(ArgAction::SetTrue))

    .arg(Arg::new("print0")
        .short('0')
        .long("print0")
        .help("Null-terminate lines")
        .help_heading("Output")
        .action(ArgAction::SetTrue))

    .arg(Arg::new("output")
        .short('o')
        .long("output")
        .help("Output file")
        .help_heading("Output")
        .group("global"))

    .arg(Arg::new("files")
        .action(ArgAction::Append))

    .after_help("Selectors can be chained using '!'.

The first selector is applied to the document root, and subsequent selectors
are applied to the results of the previous selector. The --parent flag can be
used to select the input element rather than the matched children.")

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
    let mut positional_args : Vec<&String> = matches.get_many::<String>("files")
        .unwrap_or_default()
        .filter(|s| *s != "!" && *s != "|").collect();
    let noprefix = *matches.get_one::<bool>("no-prefix").unwrap_or(&false);
    let prefix = (*matches.get_one::<bool>("prefix").unwrap_or(&false) || positional_args.len() > 1) && !noprefix;
    let out_opt = matches.get_one::<String>("output");
    let list = *matches.get_one::<bool>("list").unwrap_or(&false);
    let quiet = *matches.get_one::<bool>("quiet").unwrap_or(&false);
    let text = *matches.get_one::<bool>("text").unwrap_or(&false);
    let trim = *matches.get_one::<bool>("trim").unwrap_or(&false);
    let count = *matches.get_one::<bool>("count").unwrap_or(&false);
    let attributes : Vec<&String> = matches.get_many::<String>("attribute").unwrap_or_default().collect();

    if positional_args.len() == 0 {
        positional_args.push(&stdin_path);
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

    let mut opts = Opts{
        attributes: attributes,
        list: list,
        quiet: quiet,
        text: text,
        prefix: prefix,
        trim: trim,
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

        let direction = if *matches.get_one::<bool>("parent").unwrap_or(&false)
            { SelectorDirection::Document }
            else { SelectorDirection::Current };

        let selectors : Vec<Selector> = matches.get_many::<String>("selector").unwrap_or_default().map(|s| {
            match Selector::parse(s) {
                Ok(sel) => sel,
                Err(_) => {
                    eprintln!("Invalid selector: {}", s);
                    exit(1);
                }
            }
        }).collect();


        let search : Vec<String> = matches.get_many::<String>("search").unwrap_or_default().map(|s| s.clone()).collect();
        let regex : Vec<Regex> = matches.get_many::<String>("regex").unwrap_or_default().map(|s| {
            match Regex::new(s) {
                Ok(re) => re,
                Err(_) => {
                    eprintln!("Invalid regex: {}", s);
                    exit(1);
                }
            }
        }).collect();

        selector_sets.push(SelectorSet {
            direction: direction,
            selectors: selectors,
            search: search,
            regex: regex,
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
