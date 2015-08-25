use std::io::prelude::*;
use std::fs::File;
use std::cell::RefCell;
use std::collections::{HashSet, HashMap};

use syntax::ast::*;
use syntax::print;
use syntax::ptr::P;
use syntax::ext;
use syntax::ext::base::ExtCtxt;
use syntax::ext::build::AstBuilder;
use syntax::parse::token::{str_to_ident, Token, IdentStyle};
use syntax::codemap::{ExpnId, ExpnInfo, CompilerExpansion, NameAndSpan, DUMMY_SP};

use rustc::session::{build_session,Session};
use rustc::session::config::{build_session_options,build_configuration, Input};
use rustc_driver::{handle_options,diagnostics_registry};
use rustc_driver::driver::{phase_1_parse_input, source_name};

use super::state::State;


const CRATE_SRC : &'static str  =  "//Generated by hsm-gen, modifications will be lost when regenerating
                                    use hsm;
                                    use super::hsm_shared_data::SharedData;
                                    use super::hsm_functions;";
const FN_CRATE_SRC: &'static str = "//Generated by hsm-gen, modifications will be lost when regenerating
                                    use hsm;
                                    use super::hsm_shared_data::SharedData;
                                    use super::hsm_generated::Events;";

pub struct HsmGenerator {
    sess        : Session,
    cfg         : CrateConfig,
    src_name    : String,
    krate       : RefCell<Crate>
}
impl HsmGenerator {
    pub fn new(prefix: bool) -> Self {
        let matches = match handle_options(vec!["".to_string(), "-".to_string()]) {
            Some(matches) => matches,
            None => unreachable!()
        };
        let descriptions = diagnostics_registry();
        let sopts = build_session_options(&matches);
        let sess = build_session(sopts, None, descriptions);
        let cfg = build_configuration(&sess);
        let input = Input::Str( if prefix {CRATE_SRC.to_string()} else {FN_CRATE_SRC.to_string()});
        let src_name = source_name(&input);
        let krate = RefCell::new(phase_1_parse_input(&sess, cfg.clone(), &input));

        HsmGenerator {
            sess    : sess,
            cfg     : cfg,
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

    pub fn print(&self, file: &str) {
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
                                        false).expect("Could not format crate");
        }
        // print!("{}", String::from_utf8(out).unwrap());
        let mut f = File::create(file).expect("Could not create file");
        f.write(&out).expect("Could not write to file");
    }

    pub fn create_event_enum(&self, hs: &HashSet<String>) {
        let cx = self.extctxt();
        let mut variants: Vec<P<Variant>> = Vec::new();
        for var_name in hs {
            let mut variant = cx.variant(
                DUMMY_SP,
                str_to_ident(var_name),
                Vec::new()
            );
            variant.node.vis = Visibility::Inherited;
            variants.push(P(variant));
        }
        let en = cx.item_enum(
            DUMMY_SP,
            str_to_ident("Events"),
            EnumDef{ variants: variants }
        ).map(|mut x| {
            x.vis = Visibility::Public;
            x.attrs.push(quote_attr!(&cx, #[derive(Debug, Clone)]));
            x
        });
        self.krate.borrow_mut().module.items.push(en);
    }

    pub fn create_hsm_objects(&self, states: &HashSet<String>) {
        let cx = self.extctxt();
        let events = str_to_ident("Events");
        let st_str = str_to_ident("StateStruct");
        let st     = str_to_ident("States");
        let shr_dat= str_to_ident("SharedData");
        let states: Vec<TokenTree> = states
            .iter()
            .map(|st_nam| vec![Token::Ident(str_to_ident(st_nam), IdentStyle::Plain)])
            .collect::<Vec<_>>()
            .join(&Token::Comma)
            .into_iter()
            .map(|t| TtToken(DUMMY_SP, t))
            .collect();

        let x = quote_item!(&cx,
            hsm_define_objects!($st_str, $st, $events, $shr_dat, (
                $states
            ));
        ).unwrap();
        self.krate.borrow_mut().module.items.push(x);
    }

    pub fn create_state_parent_impls(&self, hm: &HashMap<String, State>) {
        let cx = self.extctxt();
        let st = str_to_ident("States");
        let par_lst: Vec<TokenTree> = hm.values()
            .map(|state| vec![
                Token::Ident(str_to_ident(state.name.as_ref().unwrap()), IdentStyle::Plain),
                Token::RArrow,
                Token::Ident(str_to_ident(if let Some(ref x) = state.parent {x} else {"None"} ), IdentStyle::Plain)
            ])
            .collect::<Vec<_>>()
            .join(&Token::Comma)
            .into_iter()
            .map(|t| TtToken(DUMMY_SP, t))
            .collect();
        let x = quote_item!(&cx,
            hsm_state_parents!($st; $par_lst);
        ).unwrap();
        self.krate.borrow_mut().module.items.push(x);
    }

    pub fn create_state_impls(&self, hm: &HashMap<String, State>) {
        let events = str_to_ident("Events");
        let states = str_to_ident("States");
        let shr_dat= str_to_ident("SharedData");
        for state in hm.values() {
            self.create_state_impl(state, &events, &states, &shr_dat);
        }
    }

    fn create_enter_exit_arm(&self, cx: &ExtCtxt, opt_func: &Option<String>, evt_type: &str) -> Option<Arm> {
        if let &Some(ref func_nam) = opt_func {
            let func_ident = str_to_ident(func_nam);
            let evt_ident = str_to_ident(evt_type);
            Some(cx.arm(DUMMY_SP,
                   vec!(quote_pat!(&cx, hsm::Event::$evt_ident)),
                   quote_expr!(&cx, {
                        hsm_functions::$func_ident(shr_data, evt);
                        hsm::Action::Ignore
                   })
            ))
        } else { None }
    }

    fn create_state_impl(&self, state: &State, events: &Ident, states: &Ident, shr_dat: &Ident) {
        let cx = self.extctxt();
        let state_ident = str_to_ident(state.name.as_ref().unwrap());
        let mut arms: Vec<Arm> = Vec::new();
        self.create_enter_exit_arm(&cx, &state.entry, "Enter").map(|x| arms.push(x));
        self.create_enter_exit_arm(&cx, &state.exit, "Exit").map(|x| arms.push(x));
        for (trigger, target) in state.transitions.iter() {
            let target_ident = str_to_ident(target);
            let trigger_ident = str_to_ident(trigger);
            let pat = if trigger == "_" {
                quote_pat!(&cx, $trigger_ident)
            } else {
                quote_pat!(&cx, hsm::Event::User($events::$trigger_ident))
            };
            let trans = quote_expr!(&cx, $states::$target_ident);
            arms.push(cx.arm(DUMMY_SP,
                             vec!(pat),
                             quote_expr!(&cx, hsm_delayed_transition!(probe, { $trans }))
            ));
        };
        if !state.transitions.contains_key("_") {
            arms.push(cx.arm(DUMMY_SP,
                             vec!(quote_pat!(&cx, _)),
                             quote_expr!(&cx, hsm::Action::Parent)
            ));
        };
        let match_expr = cx.expr_match(DUMMY_SP, quote_expr!(&cx, evt), arms);
        let x = quote_item!(&cx,
            impl hsm::State<$events, $states, $shr_dat> for $state_ident {
                fn handle_event(&mut self, shr_data: &mut $shr_dat, evt: hsm::Event<$events>, probe: bool) -> hsm::Action<$states> {
                    $match_expr
                }
            }
        ).unwrap();
        self.krate.borrow_mut().module.items.push(x);
    }

    pub fn create_function_stubs(&self, hm: &HashMap<String, State>) {
        let cx = self.extctxt();
        let events = str_to_ident("Events");
        let shr_dat= str_to_ident("SharedData");
        let mut functions = HashSet::new();
        hm.values().map(|s| {
            s.entry.as_ref().map(|x|
                functions.insert(x.to_string())
            );
            s.exit.as_ref().map(|x|
                functions.insert(x.to_string())
            )
        }).count();
        for func in functions {
            let func_ident  = str_to_ident(&func);
            let it = quote_item!(&cx,
                fn $func_ident(shr_data: &mut $shr_dat, evt: hsm::Event<$events>) {
                    unimplemented!();
                }
            ).unwrap().map(|mut x| {
                x.vis = Visibility::Public;
                x
            });
            self.krate.borrow_mut().module.items.push(it);
            // println!("{}", func);
        }
    }
}
