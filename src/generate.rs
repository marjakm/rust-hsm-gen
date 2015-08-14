use std::io::Write;
use std::cell::RefCell;

use syntax::ast;
use syntax::parse;
use syntax::ext;
use syntax::print;
use syntax::ext::base::ExtCtxt;
use syntax::codemap::{ExpnId, ExpnInfo, CompilerExpansion, NameAndSpan};

use rustc::session::{build_session,Session};
use rustc::session::config::{build_session_options,build_configuration, Input};
use rustc_driver::{handle_options,diagnostics_registry};
use rustc_driver::driver::{phase_1_parse_input, source_name};

const crate_src : &'static str =  "fn main(){}";

pub struct HsmGenerator {
    sess        : Session,
    cfg         : ast::CrateConfig,
    input       : Input,
    src_name    : String,
    krate       : RefCell<ast::Crate>
}
impl HsmGenerator {
    pub fn new() -> Self {
        let matches = match handle_options(vec!["".to_string(), "-".to_string()]) {
            Some(matches) => matches,
            None => unreachable!()
        };
        let descriptions = diagnostics_registry();
        let sopts = build_session_options(&matches);
        let sess = build_session(sopts, None, descriptions);
        let cfg = build_configuration(&sess);
        let input = Input::Str(crate_src.to_string());
        let src_name = source_name(&input);
        let mut krate = RefCell::new(phase_1_parse_input(&sess, cfg.clone(), &input));

        HsmGenerator {
            sess    : sess,
            cfg     : cfg,
            input   : input,
            src_name: src_name,
            krate   : krate
        }
    }

    fn ecfg(&self) -> ext::expand::ExpansionConfig {
        ext::expand::ExpansionConfig {
            crate_name      : self.src_name.clone(),
            features        : None, //Some(&self.sess.features.borrow()),
            recursion_limit : self.sess.recursion_limit.get(),
            trace_mac       : self.sess.opts.debugging_opts.trace_macros,
        }
    }
    fn extctxt(&self) -> ExtCtxt {
        let mut cx = ExtCtxt::new(&self.sess.parse_sess, self.cfg.clone(), self.ecfg());
        cx.backtrace = ExpnId::from_u32(0);
        cx.codemap().record_expansion(
            ExpnInfo {
                call_site: self.krate.borrow().span,
                callee: NameAndSpan {
                    name: "mattis".to_string(),
                    format: CompilerExpansion,
                    allow_internal_unstable: true,
                    span: None,
                },
            }
        );
        cx
    }

    pub fn print(&self) {
        let src_nam = self.src_name.clone();
        let src = self.sess.codemap().get_filemap(&src_nam[..])
                            .src
                            .as_ref()
                            .unwrap()
                            .as_bytes()
                            .to_vec();
        let mut rdr = &src[..];
        let mut out = Vec::new();
        let ann = print::pprust::NoAnn;
        {
            let out_w: &mut Write = &mut out;
            print::pprust::print_crate( self.sess.codemap(),
                                        self.sess.diagnostic(),
                                        &self.krate.borrow(),
                                        src_nam,
                                        &mut rdr,
                                        box out_w,
                                        &ann,
                                        false);
        }
        print!("{}", String::from_utf8(out).unwrap());
    }


    pub fn test_modification(&self) {
        let cx = self.extctxt();
        let x = quote_item!(&cx,
            fn foo() -> u32 {
                let sum = 2 + 2;
                sum
            }
        ).unwrap();
        self.krate.borrow_mut().module.items.push(x);
    }
}
