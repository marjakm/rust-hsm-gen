use sxd_document::Package;
use sxd_document::dom::Document;
use sxd_document::writer::format_document;
use sxd_document::parser::Parser;

use sxd_xpath;

use std::io::prelude::*;
use std::fs::File;

pub struct XmiReader {
    package : Package
}
impl XmiReader {
    pub fn new() -> Self {
        XmiReader {
            package : Package::new()
        }
    }

    pub fn from_file(file: &str) -> Self {
        let mut f = File::open(file).expect("No such file");
        let mut s = String::new();
        f.read_to_string(&mut s).expect("Could not read to string");
        let parser = Parser::new();
        let parseres = parser.parse(&s);
        println!("{:#?}", parseres);
        XmiReader {
            package: parseres.ok().expect("Could not parse")
        }
    }

    fn doc(&self) -> Document {
        self.package.as_document()
    }

    pub fn print(&mut self, file: &str) {
        let mut f = File::create(file).expect("Could not create file");
        let doc = self.doc();
        format_document(&doc, &mut f).ok().expect("unable to output XML");
    }

    pub fn test(&self) {
        let doc = self.doc();

        let hello = doc.create_element("hello");
        hello.set_attribute_value("planet", "Earth");
        let comment = doc.create_comment("What about other planets?");
        let text = doc.create_text("Greetings, Earthlings!");

        hello.append_child(comment);
        hello.append_child(text);
        doc.root().append_child(hello);
        info!("{:?}", doc);
    }
}
