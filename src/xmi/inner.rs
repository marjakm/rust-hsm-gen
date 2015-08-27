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
