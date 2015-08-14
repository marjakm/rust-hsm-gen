#![feature(quote, rustc_private)]
#![feature(box_syntax)]

extern crate syntax;
extern crate rustc;
extern crate rustc_driver;

mod generate;

pub use generate::HsmGenerator;
