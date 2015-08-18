#[macro_use]
extern crate log;
extern crate fern;
extern crate time;

extern crate hsm_gen;
extern crate clap;
extern crate sxd_xpath;

use clap::{App, Arg};


fn main() {
    conf_logger();
    let (inp, outp) = get_options();
    let xmireader = hsm_gen::XmiReader::from_file(&inp);
    // xmireader.test();
    // xmireader.print(&outp);

    let states = xmireader.states();
    // println!("states: {:?}", states);
    let events = xmireader.events();
    // println!("events: {:?}", events);
    let state_impls = xmireader.state_impls();
    // println!("state_impls: {:#?}", state_impls);

    let generator = hsm_gen::HsmGenerator::new();
    generator.create_event_enum(events);
    generator.create_hsm_objects(states);
    generator.create_state_impls(state_impls);
    generator.print(&outp);
}

fn get_options() -> (String, String) {
    let matches = App::new("HSM Generator")
                  .version("0.1.0")
                  .author("Mattis Marjak <mattis.marjak@gmail.com>")
                  .about("Generates HSM source from xmi files")
                  .arg(Arg::with_name("INPUT")
                       .short("i")
                       .help("Sets the input file to use")
                       .required(true)
                       .takes_value(true))
                  .arg(Arg::with_name("OUTPUT")
                       .short("o")
                       .help("Saves output to this file")
                       .required(true)
                       .takes_value(true))
                       .get_matches();
    (matches.value_of("INPUT").unwrap().to_string(),
     matches.value_of("OUTPUT").unwrap().to_string())
}

fn conf_logger() {
    let logger_config = fern::DispatchConfig {
        format: Box::new(|msg: &str, level: &log::LogLevel, _location: &log::LogLocation| {
            let t = time::now();
            let ms = t.tm_nsec/1000_000;
            format!("{}.{:3} [{}] {}", t.strftime("%Y-%m-%dT%H:%M:%S").unwrap(), ms, level, msg)
        }),
        output: vec![fern::OutputConfig::stderr()],
        level: log::LogLevelFilter::Trace,
    };

    if let Err(e) = fern::init_global_logger(logger_config, log::LogLevelFilter::Trace) {
        panic!("Failed to initialize global logger: {}", e);
    }
}
