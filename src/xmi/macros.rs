macro_rules! get_ns {
    ($slf:ident, $e:expr) => {
        if let Value::Nodeset(nodeset) = $slf.evaluate_root($e) {
            nodeset
        } else {
            panic!("Get ns 1 panic");
        }
    };
    ($slf:ident, $n:ident, $e:expr) => {
        if let Value::Nodeset(nodeset) = $slf.evaluate($n, $e) {
            nodeset
        } else {
            panic!("Get ns 2 panic");
        }
    }
}

macro_rules! get_attr_val_str {
    ($i:ident) => {
        if let Node::Attribute(x) = $i {
            x.value().to_string()
        } else {
            panic!("get_attr_val_str");
        }
    }
}

macro_rules! get_attrs {
    ($i:ident) => {
        if let Node::Element(x) = $i {
            x.attributes()
        } else {
            panic!("get_attrs");
        }
    }
}
