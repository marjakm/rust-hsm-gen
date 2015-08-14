#![feature(quote, rustc_private)]
#![feature(box_syntax)]
#![feature(result_expect)]


#[macro_use]
extern crate log;
extern crate syntax;
extern crate rustc;
extern crate rustc_driver;
extern crate sxd_document;
extern crate sxd_xpath;

mod generate;
mod xmi;

pub use generate::HsmGenerator;
pub use xmi::XmiReader;
