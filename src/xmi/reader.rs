use std::io::prelude::*;
use std::fs::File;
use std::collections::HashMap;
use std::collections::HashSet;

use sxd_document::Package;
use sxd_document::writer::format_document;
use sxd_document::parser::Parser;
use sxd_xpath::{Value,Functions,Variables,Namespaces,Factory,EvaluationContext,Expression};
use sxd_xpath::function::register_core_functions;
use sxd_xpath::nodeset::Node;

use super::super::state::State;
use super::inner::InnerXmiReader;


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
        // println!("{:#?}", parseres);
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

    pub fn state_impls(&'a self) -> HashMap<String, State> {
        let mut v = HashMap::new();
        for state_node in get_ns!(self, "//subvertex[@name]").iter() {
            let mut state = State::new();
            state.name = Some(self.get_attr(state_node, "name"));
            if let Some(entry_node) = get_ns!(self, state_node, "entry").iter().next() {
                state.entry = Some(self.get_attr(entry_node, "name"))
            }
            if let Some(exit_node) = get_ns!(self, state_node, "exit").iter().next() {
                state.exit = Some(self.get_attr(exit_node, "name"))
            }

            let gp_node = state_node.parent().unwrap().parent().unwrap();
            if let Node::Element(gp_elem) = gp_node {
                if gp_elem.name().local_part() == "subvertex" {
                    state.parent = Some(self.get_attr(gp_node, "name"));
                }
            };

            v.insert(self.get_attr(state_node, "id"), state);
        }
        let mut nam_map = HashMap::new();
        v.iter().map(|(id, state)| nam_map.insert(
            id.to_string(),
            state.name.as_ref().unwrap().to_string()
        )).count();
        for (id, state) in v.iter_mut() {
            for transition in get_ns!(self, &format!("//transition[@source='{}']", id)) {
                if let Some(target) = nam_map.get(&self.get_attr(transition, "target")) {
                    for trigger in get_ns!(self, transition, "trigger") {
                        state.transitions.insert(
                            self.get_attr(trigger, "name"),
                            target.to_string()
                        );
                    }
                } else {
                    debug!("TODO: Handle initial and final states");
                }
            }
        }
        v
    }

    pub fn events(&'a self) -> HashSet<String> {
        let mut v = HashSet::new();
        for node in get_ns!(self, "//trigger/@name").iter() {
            v.insert(get_attr_val_str!(node));
        }
        v.remove("_");
        v
    }

    pub fn states(&'a self) -> HashSet<String> {
        let mut v = HashSet::new();
        for node in get_ns!(self, "//subvertex/@name").iter() {
            v.insert(get_attr_val_str!(node));
        }
        v
    }

    fn get_attr(&self, node: Node, attr: &str) -> String {
        for a in get_attrs!(node).iter() {
            if a.name().local_part() == attr {
                return a.value().to_string()
            }
        }
        unreachable!()
    }

    pub fn print_node(&self, node: Node) {
        println!("{:#?}", node);
        println!("{:#?}", node.children());
        for attr in get_attrs!(node) {
            println!("{:#?}={:#?}", attr.name().local_part(), attr.value());
        }
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
