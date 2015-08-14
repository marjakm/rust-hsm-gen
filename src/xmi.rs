use std::io::prelude::*;
use std::fs::File;
use std::collections::HashMap;

use sxd_document::Package;
use sxd_document::dom::Document;
use sxd_document::writer::format_document;
use sxd_document::parser::Parser;
use sxd_xpath::{Value,Functions,Variables,Namespaces,Factory,EvaluationContext,Expression};
use sxd_xpath::function::register_core_functions;

pub struct XmiReader<'a> {
    package     : Package,
    functions   : Functions,
    variables   : Variables<'a>,
    namespaces  : Namespaces,
    factory     : Factory,
}
impl<'a> XmiReader<'a> {
    pub fn new() -> Self {
        let mut fns = HashMap::new();
        register_core_functions(&mut fns);
        XmiReader {
            package     : Package::new(),
            functions   : fns,
            variables   : HashMap::new(),
            namespaces  : HashMap::new(),
            factory     : Factory::new(),
        }
    }

    pub fn from_file(file: &str) -> Self {
        let mut f = File::open(file).expect("No such file");
        let mut s = String::new();
        f.read_to_string(&mut s).expect("Could not read to string");
        let parser = Parser::new();
        let parseres = parser.parse(&s);
        println!("{:#?}", parseres);
        let mut xmireader = XmiReader::new();
        xmireader.package = parseres.ok().expect("Could not parse");
        xmireader
    }

    fn evaluate(&self, xpath: &str) -> Value {
        let root = self.doc().root();
        let context = EvaluationContext::new(
            root,
            &self.functions,
            &self.variables,
            &self.namespaces,
        );
        let xpath = self.factory.build(xpath).unwrap().unwrap();
        xpath.evaluate(&context).ok().unwrap()
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
