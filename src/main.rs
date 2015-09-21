/*
 * The MIT License (MIT)
 *
 * Copyright (c) 2015 Mattis Marjak (mattis.marjak@gmail.com)
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */
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
    let (inp, outp, fstub) = get_options();
    let xmireader = hsm_gen::XmiReader::from_file(&inp);
    // xmireader.test();
    // xmireader.print(&outp);

    let states = xmireader.states();
    // println!("states: {:?}", states);
    let events = xmireader.events();
    // println!("events: {:?}", events);
    let state_impls = xmireader.state_impls();
    // println!("state_impls: {:#?}", state_impls);

    // let mut generator = hsm_gen::HsmGenerator::new(true);
    // generator.create_event_enum(&events);
    // generator.create_hsm_objects(&states);
    // generator.create_state_parent_impls(&state_impls);
    // generator.create_state_impls(&state_impls);
    // generator.print(&outp);
    // if let Some(fstubfle) = fstub {
    //     let mut gen2 = hsm_gen::HsmGenerator::new(false);
    //     gen2.create_function_stubs(&state_impls);
    //     gen2.print(&fstubfle);
    // }
}

fn get_options() -> (String, String, Option<String>) {
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
                      .arg(Arg::with_name("FUNC_STUBS")
                       .short("f")
                       .help("Writes function stubs to this file")
                       .required(false)
                       .takes_value(true))
                  .get_matches();
    (matches.value_of("INPUT").unwrap().to_string(),
     matches.value_of("OUTPUT").unwrap().to_string(),
     matches.value_of("FUNC_STUBS").map(|x| x.to_string()))
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
