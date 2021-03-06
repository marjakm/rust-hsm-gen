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
use std::io::prelude::*;
use std::fs::File;
use std::collections::HashMap;

use sxd_document::Package;
use sxd_document::writer::format_document;
use sxd_document::parser::Parser;
use sxd_xpath::{Value,Functions,Variables,Namespaces,Factory,EvaluationContext,Expression};
use sxd_xpath::function::register_core_functions;
use sxd_xpath::nodeset::Node;

use ::ir::{State, Subvertex};
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

    pub fn read_states(&'a self) -> HashMap<String, State> {
        let mut sm = HashMap::new();
        let mut vm = HashMap::new();

        let subvertexes = get_ns!(self, "//subvertex").iter()
                                .map(|x| Subvertex::from_xml(self, x))
                                .filter(|x| x.is_some())
                                .map(|x| x.unwrap())
                                .collect::<Vec<Subvertex>>();
        // debug!("{:#?}", subvertexes);
        subvertexes.into_iter().map(|subvertex|
            match subvertex {
                Subvertex::Initial  {ref id}                   => { vm.insert(id.clone(), subvertex.clone() ); },
                Subvertex::Final    {ref id}                   => { vm.insert(id.clone(), subvertex.clone() ); },
                Subvertex::State    {ref id, ref state}        => { sm.insert(id.clone(), state.clone()     ); },
                Subvertex::Junction {ref id, ref transition}   => { vm.insert(id.clone(), subvertex.clone() ); },
                Subvertex::Choice   {ref id, ref transitions}  => { vm.insert(id.clone(), subvertex.clone() ); },
            }
        ).count();
        // debug!("{:#?}", sm);
        // Convert transitions to condactions
        for key in sm.keys().map(|x| x.to_string()).collect::<Vec<String>>().iter() {
            let mut state = sm.get(key).unwrap().clone();
            while let Some(trans) = state.transitions.pop() {
                state.add_action(trans, &sm, &vm);
            }
            sm.insert(key.to_string(), state);
        }
        // debug!("{:#?}", sm);
        // Replace hashmap keys with state names
        for key in sm.keys().map(|x| x.to_string()).collect::<Vec<String>>().iter() {
            let state = sm.remove(key).unwrap();
            sm.insert(state.name.clone(), state);
        }
        // debug!("{:#?}", sm);
        sm
    }

    pub fn parent_state_node(&'a self, node: Node<'a>) -> Option<Node<'a>> {
        let gp_node = node.parent().unwrap().parent().unwrap();
        if let Node::Element(gp_elem) = gp_node {
            if gp_elem.name().local_part() == "subvertex" {
                return Some(gp_node)
            }
        }
        None
    }

    pub fn get_attr(&self, node: Node, attr: &str) -> Option<String> {
        for a in get_attrs!(node).iter() {
            if a.name().local_part() == attr {
                return Some(a.value().to_string())
            }
        }
        None
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
}
