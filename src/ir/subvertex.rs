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
use sxd_xpath::nodeset::Node;
use ::XmiReader;
use super::{State, Transition};


#[derive(Debug, Clone)]
pub enum Subvertex {
    Initial         {id: String},
    Final           {id: String},
    State           {id: String, state:        State},
    Junction        {id: String, transition:   Transition},
    Choice          {id: String, transitions:  Vec<Transition>},
}

impl Subvertex {
    pub fn from_xml(reader: &XmiReader, node: Node) -> Option<Self> {
        let id = reader.get_attr(node, "id").unwrap();
        match reader.get_attr(node, "type").unwrap().as_str() {
            "uml:State"       => Some(Subvertex::State {id: id, state: State::from_xml(reader, node)}),
            "uml:FinalState"  => Some(Subvertex::Final {id: id}),
            "uml:Pseudostate" => {
                if let Some(kind) = reader.get_attr(node, "kind") {
                    match kind.as_str() {
                        "junction" => Some(Subvertex::Junction {
                            id:         id.clone(),
                            transition: Transition::from_xml(
                                reader,
                                get_node!(reader, &format!("//transition[@source='{}']", id))
                            )
                        }),
                        "choice"   => {
                            let mut transitions = Vec::new();
                            for trans_node in get_ns!(reader, &format!("//transition[@source='{}']", id)) {
                                transitions.push(Transition::from_xml(reader, trans_node));
                            }
                            Some(Subvertex::Choice {id: id, transitions: transitions})
                        },
                        _ => panic!("Pseudostate with unknown type")
                    }
                } else {
                    match reader.parent_state_node(node).map(|x| reader.get_attr(x, "id").expect("Subvertex parent state without id")) {
                        Some(p_id) => None,
                        None       => Some(Subvertex::Initial {id: id})
                    }
                }
            },
            _ => panic!("subvertex with unknown type")
        }
    }
}
