use std::mem;
use std::io::prelude::*;
use std::fs::File;
use std::collections::HashMap;

use sxd_document::Package;
use sxd_document::dom::Document;
use sxd_document::writer::format_document;
use sxd_document::parser::Parser;
use sxd_xpath::{Value,Functions,Variables,Namespaces,Factory,EvaluationContext,Expression};
use sxd_xpath::function::register_core_functions;
use sxd_xpath::nodeset::Node;


struct InnerXmiReader <'d> {
    package : Package,
    doc     : Document<'d>,
}
impl<'d> InnerXmiReader<'d> {
    pub fn new(pkg: Package) -> Self {
        InnerXmiReader {
            package : pkg,
            doc     : unsafe {mem::uninitialized()},
        }
    }
    pub fn init(&self) {
        unsafe {
            let mut dest: *mut Document = mem::transmute(&self.doc);
            *dest = self.package.as_document();
        }
    }
    pub fn doc(&self) -> &Document<'d> {
        &self.doc
    }
}


pub struct XmiReader<'a, 'd> {
    inner       : Box<InnerXmiReader<'d>>,
    functions   : Functions,
    variables   : Variables<'a>,
    namespaces  : Namespaces,
    factory     : Factory,
}
impl<'a:'d, 'd> XmiReader<'a, 'd> {
    fn _new(pkg: Package) -> Self {
        let mut fns = HashMap::new();
        register_core_functions(&mut fns);
        let xr = XmiReader {
            inner       : Box::new(InnerXmiReader::new(pkg)),
            functions   : fns,
            variables   : HashMap::new(),
            namespaces  : HashMap::new(),
            factory     : Factory::new(),
        };
        xr.inner.init();
        xr
    }

    pub fn new() -> Self {
        XmiReader::_new(Package::new())
    }

    pub fn from_file(file: &str) -> Self {
        let mut f = File::open(file).expect("No such file");
        let mut s = String::new();
        f.read_to_string(&mut s).expect("Could not read to string");
        let parser = Parser::new();
        let parseres = parser.parse(&s);
        println!("{:#?}", parseres);
        XmiReader::_new(parseres.ok().expect("Could not parse"))
    }

    pub fn evaluate_root(&'a self, xpath: &str) -> Value<'d>{
        self.evaluate(self.inner.doc().root(), xpath)
    }

    pub fn evaluate<N>(&'a self, node: N, xpath: &str) -> Value<'d>
        where N: Into<Node<'d>> {
        let context = EvaluationContext::new(
            node,
            &self.functions,
            &self.variables,
            &self.namespaces,
        );
        let xpath = self.factory.build(xpath).unwrap().unwrap();
        xpath.evaluate(&context).ok().unwrap()
    }

    pub fn print(&'a self, file: &str) {
        let doc = self.inner.doc();
        let mut f = File::create(file).expect("Could not create file");
        println!("{:?}", doc);
        format_document(doc, &mut f).expect("Error formatting document");
    }

    pub fn test(&'a self) {
        let doc = self.inner.doc();
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
