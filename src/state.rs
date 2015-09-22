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
use std::collections::HashMap;
use super::xmi::XmiReader;
use sxd_xpath::nodeset::Node;

pub enum Subvertex {
    Initial         {id: String},
    Final           {id: String},

    StateInitial    {id: String, parent_id:    String},
    State           {id: String, state:        State},
    Junction        {id: String, transition:   Transition},
    Choice          {id: String, transitions:  Vec<Transition>},
}
impl Subvertex {
    pub fn from_xml(reader: &XmiReader, node: Node) -> Self {
        let id = reader.get_attr(node, "id").unwrap();
        match reader.get_attr(node, "type").unwrap().as_str() {
            "uml:State"       => Subvertex::State {id: id, state: State::from_xml(reader, node)},
            "uml:FinalState"  => Subvertex::Final {id: id},
            "uml:Pseudostate" => {
                if let Some(kind) = reader.get_attr(node, "kind") {
                    match kind.as_str() {
                        "junction" => Subvertex::Junction {
                            id:         id.clone(),
                            transition: Transition::from_xml(
                                reader,
                                get_node!(reader, &format!("//transition[@source='{}']", id))
                            )
                        },
                        "choice"   => {
                            let mut transitions = Vec::new();
                            for trans_node in get_ns!(reader, &format!("//transition[@source='{}']", id)) {
                                transitions.push(Transition::from_xml(reader, trans_node));
                            }
                            Subvertex::Choice {id: id, transitions: transitions}
                        },
                        _ => panic!("Pseudostate with unknown type")
                    }
                } else {
                    match reader.parent_state_node(node).map(|x| reader.get_attr(x, "name").unwrap()) {
                        Some(p_id) => Subvertex::StateInitial {id: id, parent_id: p_id},
                        None       => Subvertex::Initial {id: id}
                    }
                }
            },
            _ => panic!("subvertex with unknown type")
        }
    }
}

pub struct Transition {
    pub source_id: String,
    pub target_id: String,
    pub guard:     Option<String>,
    pub effect:    Option<String>,
    pub trigger:   Option<Event>,
}
impl Transition {
    pub fn from_xml(reader: &XmiReader, node: Node) -> Self {
        let guard = get_node_opt!(reader, node, "//ownedRule/specification").map(|x| reader.get_attr(x, "value").unwrap());
        let effect = get_node_opt!(reader, node, "//effect/body").map(|x| x.string_value());
        let trigger = get_node_opt!(reader, node, "//trigger").map(|trig_node| Event::from_xml(reader,
            get_node!(reader, &format!("//packagedElement[@id='{}']", reader.get_attr(trig_node, "event").unwrap()))
        ));
        Transition {
            source_id: reader.get_attr(node, "source").unwrap(),
            target_id: reader.get_attr(node, "target").unwrap(),
            guard:     guard,
            effect:    effect,
            trigger:   trigger,
        }
    }
}

pub enum Event {
    Time   {id: String, name: String, relative: bool, timeout_ms: u64},
    Signal {id: String, name: String},
    Any,
}
impl Event {
    pub fn from_xml(reader: &XmiReader, node: Node) -> Self {
        match reader.get_attr(node, "type").unwrap().as_str() {
            "uml:TimeEvent"       => Event::Time {
                id:         reader.get_attr(node, "id").unwrap(),
                name:       reader.get_attr(node, "name").expect("TimeEvent with no name"),
                relative:   match reader.get_attr(node, "isRelative").expect("TimeEvent without isRelative").as_str() {
                    "true"  => true,
                    "false" => false,
                    _ => panic!("Relative with unknown value")
                },
                timeout_ms: u64::from_str_radix(
                    reader.get_attr(get_node!(reader, node, "when/expr"), "value")
                        .expect("TimeEvent without timeout")
                        .as_str(),
                    10
                ).unwrap()
            },
            "uml:SignalEvent"     => Event::Signal {
                id:         reader.get_attr(node, "id").unwrap(),
                name:       reader.get_attr(node, "name").expect("SignalEvent with no name")
            },
            "uml:AnyReceiveEvent" => Event::Any,
            _ => panic!("Event with unknown type")
        }
    }
}

#[derive(Debug)]
pub enum Action {
    Ignore(Option<String>),
    Parent(Option<String>),
    Transition(Option<String>, String),
}

#[derive(Debug)]
pub struct CondAction {
    pub cond:   Option<String>,
    pub action: Action
}

#[derive(Debug)]
pub struct State {
    pub name        : String,
    pub parent      : Option<String>,
    pub entry       : Option<String>,
    pub exit        : Option<String>,
    pub signals     : HashMap<String, Vec<CondAction>>
}
impl State {
    pub fn from_xml(reader: &XmiReader, node: Node) -> Self {
        State {
            name        : reader.get_attr(node, "name").unwrap(),
            parent      : reader.parent_state_node(node).map(|x| reader.get_attr(x, "name").unwrap()),
            entry       : get_node_opt!(reader, node, "entry").map(|x| reader.get_attr(x, "name").unwrap()),
            exit        : get_node_opt!(reader, node, "exit").map(|x| reader.get_attr(x, "name").unwrap()),
            signals     : HashMap::new()
        }
    }
}
