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
