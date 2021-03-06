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
use std::rc::Rc;
use syntax::ast::CrateConfig;
use syntax::ext::expand::ExpansionConfig;

use rustc::middle::cstore::DummyCrateStore;
use rustc::session::{build_session, Session};
use rustc::session::config::{build_session_options, build_configuration, Input};
use rustc_driver::{handle_options, diagnostics_registry};
use rustc_driver::driver::source_name;


const CRATE_SRC : &'static str  =  "//Generated by hsm-gen, modifications will be lost when regenerating
                                    use hsm;
                                    use time::SteadyTime;
                                    use enum_timer::{TimerEvent, TimerStorage};
                                    use super::hsm_uses::*;";
const FN_CRATE_SRC: &'static str = "//Generated by hsm-gen, modifications will be lost when regenerating
                                    use hsm;
                                    use super::hsm_uses::SharedData;";


pub struct Inner {
    pub sess     : Session,
    pub cfg      : CrateConfig,
    pub src_name : String,
    pub input    : Input,
}
impl Inner {
    pub fn new(prefix: bool) -> Self {
        let matches = match handle_options(vec!["".to_string(), "-".to_string()]) {
            Some(matches) => matches,
            None => unreachable!()
        };
        let descriptions = diagnostics_registry();
        let sopts = build_session_options(&matches);
        let sess = build_session(sopts, None, descriptions, Rc::new(DummyCrateStore));
        let cfg = build_configuration(&sess);
        let input = Input::Str( if prefix {CRATE_SRC.to_string()} else {FN_CRATE_SRC.to_string()});

        Inner {
            sess     : sess,
            cfg      : cfg,
            src_name : source_name(&input),
            input    : input,
        }
    }

    pub fn ecfg(&self) -> ExpansionConfig {
        ExpansionConfig {
            crate_name      : self.src_name.clone(),
            features        : None, //Some(&self.sess.features.borrow()),
            recursion_limit : self.sess.recursion_limit.get(),
            trace_mac       : self.sess.opts.debugging_opts.trace_macros.clone(),
        }
    }
}
