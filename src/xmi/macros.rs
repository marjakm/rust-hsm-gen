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
macro_rules! get_ns {
    ($slf:ident, $e:expr) => {
        if let ::sxd_xpath::Value::Nodeset(nodeset) = $slf.evaluate_root($e) {
            nodeset
        } else {
            panic!("get_ns macro: evaluate_root({}) did not return a Value::Nodeset", $e);
        }
    };
    ($slf:ident, $n:ident, $e:expr) => {
        if let ::sxd_xpath::Value::Nodeset(nodeset) = $slf.evaluate($n, $e) {
            nodeset
        } else {
            panic!("get_ns macro: evaluate({:?}, {}) did not return a Value::Nodeset", $n, $e);
        }
    }
}

macro_rules! get_node {
    ($slf:ident, $e:expr) => {{
        let ns = get_ns!($slf, $e);
        if ns.size() == 1 {
            ns.iter().next().unwrap()
        } else {
            panic!("get_node macro: required nodeset length 1, but got {}", ns.size());
        }
    }};
    ($slf:ident, $n:ident, $e:expr) => {{
        let ns = get_ns!($slf, $n, $e);
        if ns.size() == 1 {
            ns.iter().next().unwrap()
        } else {
            panic!("get_node macro: required nodeset length 1, but got {}", ns.size());
        }
    }}
}

macro_rules! get_node_opt {
    ($slf:ident, $e:expr) => {{
        let ns = get_ns!($slf, $e);
        match ns.size() {
            0 => None,
            1 => Some(ns.iter().next().unwrap()),
            _ => panic!("get_node_opt macro: required nodeset length 0 or 1, but got {}", ns.size())
        }
    }};
    ($slf:ident, $n:ident, $e:expr) => {{
        let ns = get_ns!($slf, $n, $e);
        match ns.size() {
            0 => None,
            1 => Some(ns.iter().next().unwrap()),
            _ => panic!("get_node_opt macro: required nodeset length 0 or 1, but got {}", ns.size())
        }
    }}
}


macro_rules! get_attr_val_str {
    ($i:ident) => {
        if let ::sxd_xpath::nodeset::Node::Attribute(x) = $i {
            x.value().to_string()
        } else {
            panic!("get_attr_val_str macro: {:?} is not Node::Attribute", $i);
        }
    }
}

macro_rules! get_attrs {
    ($i:ident) => {
        if let ::sxd_xpath::nodeset::Node::Element(x) = $i {
            x.attributes()
        } else {
            panic!("get_attrs macro: {:?} is not Node::Element", $i);
        }
    }
}
