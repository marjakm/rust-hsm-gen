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
use sxd_xpath::nodeset::Node;
use ::XmiReader;
use super::{Transition, Subvertex, Event, CondAction, Action};


#[derive(Debug, Clone)]
pub struct State {
    pub name        : String,
    pub parent      : Option<String>,
    pub entry       : Option<String>,
    pub exit        : Option<String>,
    pub actions     : HashMap<Event, Vec<CondAction>>,

    // used only in xmi reading
    pub transitions       : Vec<Transition>,
    pub initial_transition: Option<Transition>
}

impl State {
    pub fn from_xml(reader: &XmiReader, node: Node) -> Self {
        let id = reader.get_attr(node, "id").unwrap();
        let mut hm = HashMap::new();
        if let Some(mut do_activ) = get_node_opt!(reader, node, "doActivity/body").map(|x| x.string_value()) {
            let pat = [ ( "&gt;"  , ">" ),
                        ( "&lt;"  , "<" ),
                        ( "&amp;" , "&" ),
                        ( "&#39;" , "\\"),
                        ( "&quot;", "\"") ];
            for a in &pat {
                do_activ = do_activ.replace(a.0, a.1);
            }
            for group in do_activ.split("\n") {
                let evt_activ: Vec<&str> = group.split("=>").collect();
                if evt_activ.len() == 2 {
                    hm.insert(
                        match evt_activ[0].trim() {
                            "_" => Event::Any,
                            x   => Event::Signal {
                                id  : "DUMMY".to_string(),
                                name: x.to_string()
                            }
                        },
                        vec!(CondAction {
                            guard   : None,
                            effect  : Some(evt_activ[1].trim().to_string()),
                            action  : Action::Ignore,
                        })
                    );
                } else {
                    panic!("Could not split do activity into evt and activity: {:?}", evt_activ)
                }
            }
        }
        State {
            name        : reader.get_attr(node, "name").expect("State without name"),
            parent      : reader.parent_state_node(node).map(|x| reader.get_attr(x, "name").expect("State parent without name")),
            entry       : get_node_opt!(reader, node, "entry/body").map(|x| x.string_value()),
            exit        : get_node_opt!(reader, node, "exit/body").map(|x| x.string_value()),
            actions     : hm,
            transitions : get_ns!(reader, &format!("//transition[@source='{}']", id)).iter()
                                    .map(|x| Transition::from_xml(reader, x)).collect(),
            initial_transition: {
                let opts = get_ns!(reader, node, "region/subvertex").iter().filter(|x| {
                    reader.get_attr(x.clone(), "type").unwrap().as_str() == "uml:Pseudostate"
                    && reader.get_attr(x.clone(), "kind").is_none()
                }).collect::<Vec<Node>>();
                match opts.len() {
                    0 => None,
                    1 => Some(Transition::from_xml(
                        reader,
                        get_node!(reader, &format!(
                            "//transition[@source='{}']",
                            reader.get_attr(opts[0], "id").unwrap()
                    )))),
                    _ => panic!("State {} initial state options", opts.len())
                }
            }
        }
    }

    pub fn add_action(&mut self, mut t: Transition, sm: &HashMap<String, State>, vm: &HashMap<String, Subvertex>) {
        match t.trigger.take() {
            Some(evt) => {
                let ca = CondAction::from_transition(t, sm, vm);
                if let Some(cond_act_vec) = self.actions.get_mut(&evt) {
                    cond_act_vec.push(ca);
                    return
                }
                self.actions.insert(evt, vec!(ca));
                return
            },
            None => panic!("ActionMap add_transition without trigger {:?}", t)
        }
    }
}
