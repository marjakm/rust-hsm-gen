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
use super::XmiReader;
use super::event::Event;


#[derive(Debug, Clone)]
pub struct Transition {
    pub source_id: String,
    pub target_id: String,
    pub guard:     Option<String>,
    pub effect:    Option<String>,
    pub trigger:   Option<Event>,
}

impl Transition {
    pub fn from_xml(reader: &XmiReader, node: Node) -> Self {
        Transition {
            source_id: reader.get_attr(node, "source").expect("Transition without source"),
            target_id: reader.get_attr(node, "target").expect("Transition without target"),
            guard:     get_node_opt!(reader, node, "ownedRule/specification").map(|x|
                match reader.get_attr(x, "type").unwrap().as_str() {
                    "uml:OpaqueExpression" => get_node!(reader, x, "body").string_value(),
                    "uml:LiteralString"    => reader.get_attr(x, "value").expect("Transition guard without value"),
                    _ => panic!("Transition guard specification type unknown")
                }
            ),
            effect:    get_node_opt!(reader, node, "effect/body").map(|x| x.string_value()),
            trigger:   get_node_opt!(reader, node, "trigger").map(|trig_node| Event::from_xml(reader, {
                let evt_id = reader.get_attr(trig_node, "event").expect("Transition trigger without event");
                get_ns!(reader, "//packagedElement").iter().filter(|x| reader.get_attr(x.clone(), "id").unwrap() == evt_id).next().unwrap()
            })),
        }
    }
}
