use std::mem;

use sxd_document::Package;
use sxd_document::dom::Document;


pub struct InnerXmiReader <'d> {
    package : Package,
    doc     : Document<'d>,
}

impl<'d> InnerXmiReader<'d> {
    pub fn new(pkg: Package) -> Self {
        InnerXmiReader {
            package : pkg,
            doc     : unsafe {mem::uninitialized()},
        }
    }
    pub fn init(&self) {
        unsafe {
            let mut dest: *mut Document = mem::transmute(&self.doc);
            *dest = self.package.as_document();
        }
    }
    pub fn doc(&self) -> &Document<'d> {
        &self.doc
    }
}
