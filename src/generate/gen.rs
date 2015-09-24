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
use std::io::prelude::*;
use std::fs::File;
use std::collections::{HashSet, HashMap};

use syntax::ast::*;
use syntax::ptr::P;
use syntax::ext::base::ExtCtxt;
use syntax::ext::build::AstBuilder;
use syntax::print::pprust::{NoAnn, print_crate};
use syntax::parse::token::{str_to_ident, Token, IdentStyle};
use syntax::codemap::{ExpnId, ExpnInfo, ExpnFormat, CompilerExpansionFormat, NameAndSpan, DUMMY_SP};
use syntax::feature_gate::GatedCfg;
use rustc_driver::driver::phase_1_parse_input;

use ::ir::{State, Event};
use super::inner::Inner;


pub struct HsmGenerator {
    inner               : Inner,
    krate               : Crate,
    feature_gated_cfgs  : Vec<GatedCfg>,

}
impl HsmGenerator {
    pub fn new(prefix: bool) -> Self {
        let inner = Inner::new(prefix);
        let krate = phase_1_parse_input(&inner.sess, inner.cfg.clone(), &inner.input);
        HsmGenerator {
            inner              : inner,
            krate              : krate,
            feature_gated_cfgs : Vec::new(),
        }
    }

    fn extctxt(&mut self) -> ExtCtxt {
        let mut cx = ExtCtxt::new(&self.inner.sess.parse_sess,
                                  self.inner.cfg.clone(),
                                  self.inner.ecfg(),
                                  &mut self.feature_gated_cfgs);
        cx.backtrace = ExpnId::from_u32(0);
        cx.codemap().record_expansion(
            ExpnInfo {
                call_site: self.krate.span,
                callee: NameAndSpan {
                    format: ExpnFormat::CompilerExpansion(CompilerExpansionFormat::PlacementIn),
                    allow_internal_unstable: true,
                    span: None,
                },
            }
        );
        cx
    }

    pub fn print(&self, file: &str) {
        let src_nam = self.inner.src_name.clone();
        let src = self.inner.sess.codemap().get_filemap(&src_nam[..])
                            .src
                            .as_ref()
                            .unwrap()
                            .as_bytes()
                            .to_vec();
        let mut rdr = &src[..];
        let mut out = Vec::new();
        let ann = NoAnn;
        {
            let out_w: &mut Write = &mut out;
            print_crate( self.inner.sess.codemap(),
                         self.inner.sess.diagnostic(),
                         &self.krate,
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

    fn create_enum(&mut self, name: &str, vm: HashMap<String, Option<String>>) -> P<Item> {
        let cx = self.extctxt();
        let mut variants: Vec<P<Variant>> = Vec::new();
        for (var_name, opt_intern) in vm.iter() {
            let mut variant = cx.variant(
                DUMMY_SP,
                str_to_ident(var_name),
                { if let Some(int_name) = opt_intern.as_ref() {
                    vec!(cx.ty_ident(DUMMY_SP, str_to_ident(int_name)))
                } else {
                    Vec::new()
                }}
            );
            variant.node.vis = Visibility::Inherited;
            variants.push(P(variant));
        }
        cx.item_enum(
            DUMMY_SP,
            str_to_ident(name),
            EnumDef{ variants: variants }
        ).map(|mut x| {
            x.vis = Visibility::Public;
            x.attrs.push(quote_attr!(&cx, #[derive(Debug, Clone)]));
            x
        })
    }

    pub fn create_event_enum(&mut self, hm: &HashMap<String, State>) {
        let mut time_evts: HashMap<String, Option<String>> = HashMap::new();
        let mut signals  : HashMap<String, Option<String>> = HashMap::new();
        signals.insert("Timeout".to_string(), Some("Timeout".to_string()));
        hm.values().map(|x| x.actions.keys().map(|e| match *e {
            Event::Signal {ref name, ..} => {
                let nam_parts = name.split("(").collect::<Vec<&str>>();
                let int_val = {
                    if nam_parts.len() == 1 {
                        None
                    } else {
                        let int_parts = nam_parts[1].split("::").collect::<Vec<&str>>();
                        if int_parts.len() == 1 {
                            Some(nam_parts[0].to_string())
                        } else {
                            Some(int_parts[0].to_string())
                        }
                    }
                };
                signals.insert(nam_parts[0].to_string(), int_val);
            },
            Event::Time   {ref name, ..} => { time_evts.insert(name.to_string(), None); },
            _ => {},
        }).count()).count();
        // debug!("{:#?}", time_evts);
        // debug!("{:#?}", signals);
        let time_enum = self.create_enum("Timeout", time_evts);
        self.krate.module.items.push(time_enum);
        let event_enum = self.create_enum("Events", signals);
        self.krate.module.items.push(event_enum);
    }

    pub fn create_hsm_objects(&mut self, hm: &HashMap<String, State>) {
        let x = {
            let cx = self.extctxt();
            let events = str_to_ident("Events");
            let st_str = str_to_ident("StateStruct");
            let st     = str_to_ident("States");
            let shr_dat= str_to_ident("SharedData");
            let states: Vec<TokenTree> = hm
                .keys()
                .map(|st_nam| vec![Token::Ident(str_to_ident(st_nam), IdentStyle::Plain)])
                .collect::<Vec<_>>()
                .join(&Token::Comma)
                .into_iter()
                .map(|t| TtToken(DUMMY_SP, t))
                .collect();

            quote_item!(&cx,
                hsm_define_objects!($st_str, $st, $events, $shr_dat, (
                    $states
                ));
            ).unwrap()
        };
        self.krate.module.items.push(x);
    }

    pub fn create_state_parent_impls(&mut self, hm: &HashMap<String, State>) {
        let x = {
            let cx = self.extctxt();
            let st = str_to_ident("States");
            let par_lst: Vec<TokenTree> = hm.values()
                .map(|state| vec![
                    Token::Ident(str_to_ident(state.name.as_str()), IdentStyle::Plain),
                    Token::RArrow,
                    Token::Ident(str_to_ident(if let Some(ref x) = state.parent {x} else {"None"} ), IdentStyle::Plain)
                ])
                .collect::<Vec<_>>()
                .join(&Token::Comma)
                .into_iter()
                .map(|t| TtToken(DUMMY_SP, t))
                .collect();
            quote_item!(&cx,
                hsm_state_parents!($st; $par_lst);
            ).unwrap()
        };
        self.krate.module.items.push(x);
    }

    /*
    pub fn create_state_impls(&mut self, hm: &HashMap<String, State>) {
        let events = str_to_ident("Events");
        let states = str_to_ident("States");
        let shr_dat= str_to_ident("SharedData");
        for state in hm.values() {
            self.create_state_impl(state, &events, &states, &shr_dat);
        }
    }
    */

    fn create_enter_exit_arm(cx: &ExtCtxt, opt_func: &Option<String>, evt_type: &str) -> Option<Arm> {
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

    /*
    fn create_state_impl(&mut self, state: &State, events: &Ident, states: &Ident, shr_dat: &Ident) {
        let x = {
            let cx = self.extctxt();
            let state_ident = str_to_ident(state.name.as_ref().unwrap());
            let mut arms: Vec<Arm> = Vec::new();
            Self::create_enter_exit_arm(&cx, &state.entry, "Enter").map(|x| arms.push(x));
            Self::create_enter_exit_arm(&cx, &state.exit, "Exit").map(|x| arms.push(x));
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
            quote_item!(&cx,
                impl hsm::State<$events, $states, $shr_dat> for $state_ident {
                    fn handle_event(&mut self, shr_data: &mut $shr_dat, evt: hsm::Event<$events>, probe: bool) -> hsm::Action<$states> {
                        $match_expr
                    }
                }
            ).unwrap()
        };
        self.krate.module.items.push(x);
    }
    */

    pub fn create_function_stubs(&mut self, hm: &HashMap<String, State>) {
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
            let it = {
                let cx = self.extctxt();
                let func_ident  = str_to_ident(&func);
                quote_item!(&cx,
                    fn $func_ident(shr_data: &mut $shr_dat, evt: hsm::Event<$events>) {
                        unimplemented!();
                    }
                ).unwrap().map(|mut x| {
                    x.vis = Visibility::Public;
                    x
                })
            };
            self.krate.module.items.push(it);
            // println!("{}", func);
        }
    }
}
