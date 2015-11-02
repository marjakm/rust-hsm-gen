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


#[derive(Debug, Eq, PartialEq, Hash, Clone, Ord, PartialOrd)]
pub enum Event {
    Time   {id: String, name: String, relative: bool, timeout_ms: u32},
    Signal {id: String, name: String},
    UserAny,
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
                timeout_ms: u32::from_str_radix(
                    reader.get_attr(get_node!(reader, node, "when/expr"), "value")
                        .expect("TimeEvent without timeout")
                        .as_str(),
                    10
                ).expect("Event timeout_ms from_str_radix")
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
