use anyhow::{anyhow, Context, Result};
use clap::{App, Arg};
use regex::{Captures, Regex};
use std::{fs, io, process};

struct Config {
    pub input_path: Option<String>,
    pub output_path: Option<String>,
    pub exec: String,
    pub no_headers: bool,
    pub delimiter: String,
    pub quote: String,
    pub arg_regex: String,
    pub new_column_name: String,
}

fn main() -> Result<()> {
    let matches = App::new("csv-exec")
        .version("0.1.0")
        .author("niladic <git@nil.choron.cc>")
        .about("Execute a command on each record of a CSV.")
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .value_name("FILE")
                .help("Input CSV file [stdin by default]")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .value_name("FILE")
                .help("Output CSV [stdout by default]")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("exec")
                .short("e")
                .long("exec")
                .value_name("COMMAND")
                .required(true)
                .help("The command to execute")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("no-header")
                .short("n")
                .long("no-header")
                .help("Do not read the first line as a header line")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("delimiter")
                .short("d")
                .long("delimiter")
                .value_name("CHAR")
                .default_value(",")
                .help("CSV delimiter (\\t for tabs)")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("quote")
                .long("quote")
                .value_name("CHAR")
                .default_value("\"")
                .help("CSV quote")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("arg-regex")
                .long("arg-regex")
                .value_name("REGEX")
                .default_value(r"\$([0-9]+)")
                .help("Regex used to parse the column position in the command args. Syntax: https://docs.rs/regex/1.3.4/regex/index.html#syntax")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("new-column-name")
                .long("new-column-name")
                .value_name("STRING")
                .default_value("Result")
                .help("Name of the new column which contains the results")
                .takes_value(true),
        )
        .get_matches();

    let config = Config {
        input_path: matches.value_of("input").map(String::from),
        output_path: matches.value_of("output").map(String::from),
        // Note: required using clap
        exec: matches
            .value_of("exec")
            .map(String::from)
            .unwrap_or_else(String::new),
        no_headers: matches.is_present("no-headers"),
        delimiter: matches
            .value_of("delimiter")
            .map(String::from)
            .unwrap_or_else(String::new),
        quote: matches
            .value_of("quote")
            .map(String::from)
            .unwrap_or_else(String::new),
        arg_regex: matches
            .value_of("arg-regex")
            .map(String::from)
            .unwrap_or_else(String::new),
        new_column_name: matches
            .value_of("new-column-name")
            .map(String::from)
            .unwrap_or_else(String::new),
    };

    run(config)
}

fn run(config: Config) -> Result<()> {
    let reader: Box<dyn io::Read> = match config.input_path {
        None => Box::new(io::stdin()),
        Some(path) => Box::new(fs::File::open(&path).context(format!("Failed to open {}", path))?),
    };

    let writer: Box<dyn io::Write> = match config.output_path {
        None => Box::new(io::stdout()),
        Some(path) => {
            Box::new(fs::File::create(&path).context(format!("Failed to create {}", path))?)
        }
    };

    let read_one_ascii_char = |value: &str| -> Result<u8> {
        if value.bytes().count() > 1 {
            return Err(anyhow!("Value {} must be 1 ASCII character", value));
        }
        match value.chars().next() {
            None => Err(anyhow!("Missing value")),
            Some(c) => {
                if c.is_ascii() {
                    Ok(c as u8)
                } else {
                    Err(anyhow!("Value {} must be 1 ASCII character", value))
                }
            }
        }
    };

    let delimiter: u8 = if &config.delimiter == r"\t" {
        b'\t'
    } else {
        read_one_ascii_char(&config.delimiter)?
    };

    let quote: u8 = read_one_ascii_char(&config.quote)?;

    let variable_regex = Regex::new(&config.arg_regex)?;

    let cmd_and_args: Vec<String> = shell_words::split(&config.exec)?;

    let mut csv_reader = csv::ReaderBuilder::new()
        .has_headers(!config.no_headers)
        .delimiter(delimiter)
        .quote(quote)
        .from_reader(reader);

    let mut csv_writer = csv::WriterBuilder::new()
        .delimiter(delimiter)
        .quote(quote)
        .from_writer(writer);

    if !config.no_headers {
        let new_headers = csv_reader.headers()?.clone();
        csv_writer.write_record(
            new_headers
                .iter()
                .chain(vec![&*config.new_column_name].into_iter()),
        )?;
    }

    for record in csv_reader.records() {
        let mut record = record?;
        let mut args_iter = cmd_and_args.iter();
        let command = match args_iter.next() {
            None => return Err(anyhow!("No command to execute")),
            Some(command) => command,
        };
        let args = args_iter
            .map(|arg| {
                variable_regex
                    .replace_all(arg, |caps: &Captures| {
                        let record_value = caps
                            .get(1)
                            .and_then(|position| position.as_str().parse::<usize>().ok())
                            .and_then(|position| record.get(position));
                        match record_value {
                            None => "",
                            Some(value) => value,
                        }
                    })
                    .to_string()
            })
            .collect::<Vec<_>>();
        let output = process::Command::new(command)
            .args(&args)
            .output()
            .context(format!(
                "Failed to execute command {} with args {:?}",
                command, args
            ))?;

        let out = std::str::from_utf8(&output.stdout)?.trim();
        record.push_field(&out);
        csv_writer.write_record(record.iter())?;
    }
    csv_writer.flush()?;
    Ok(())
}
