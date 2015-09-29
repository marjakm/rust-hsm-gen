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

use ::ir::{State, Event, CondAction, Action};
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
            let variant = cx.variant(
                DUMMY_SP,
                str_to_ident(var_name),
                { if let Some(int_name) = opt_intern.as_ref() {
                    vec!(cx.ty_ident(DUMMY_SP, str_to_ident(int_name)))
                } else {
                    Vec::new()
                }}
            );
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

    pub fn create_state_impls(&mut self, hm: &HashMap<String, State>) {
        let events = str_to_ident("Events");
        let states = str_to_ident("States");
        let shr_dat= str_to_ident("SharedData");
        let timeout= str_to_ident("Timeout");
        let mut states_vec = hm.iter().collect::<Vec<(&String, &State)>>();
        states_vec.sort_by(|a,b| a.0.cmp(b.0));
        for state in states_vec.iter().map(|x| x.1) {
            let st_impl = self.create_state_impl(state, &events, &states, &shr_dat, &timeout);
            self.krate.module.items.push(st_impl);
        }
    }

    fn create_enter_exit_arm(cx: &ExtCtxt, opt_func: &Option<String>, evt_type: &str, extra: Vec<P<Expr>>) -> Option<Arm> {
        if let &Some(ref func_nam) = opt_func {
            let func_ident = str_to_ident(func_nam);
            let evt_ident = str_to_ident(evt_type);
            Some(cx.arm(DUMMY_SP,
                   vec!(quote_pat!(&cx, hsm::Event::$evt_ident)),
                   quote_expr!(&cx, {
                        $func_ident;
                        $extra;
                        hsm::Action::Ignore
                   })
            ))
        } else {
            if extra.len() > 0 {
                let evt_ident = str_to_ident(evt_type);
                Some(cx.arm(DUMMY_SP,
                       vec!(quote_pat!(&cx, hsm::Event::$evt_ident)),
                       quote_expr!(&cx, {
                            $extra;
                            hsm::Action::Ignore
                       })
                ))
            } else {
                None
            }
        }
    }

    fn get_condaction_expr(cx: &ExtCtxt, ca: &CondAction, states: &Ident) -> (P<Expr>, bool) {
        let mut use_delayed_transition = true;
        let action = match ca.action {
            Action::Ignore                  => {
                use_delayed_transition = false;
                quote_expr!(&cx, hsm::Action::Ignore)
            },
            Action::Parent                  => {
                use_delayed_transition = false;
                quote_expr!(&cx, hsm::Action::Parent)
            },
            Action::Transition(ref st_str)  => {
                let st = str_to_ident(st_str);
                quote_expr!(&cx, $states::$st)
            },
            Action::Diverge(ref ca_vec)     => {
                let tupl = Self::create_action_expr(cx, ca_vec, states);
                use_delayed_transition = tupl.1;
                tupl.0
            }
        };
        let effect = ca.effect.as_ref().map(|x| str_to_ident(x));
        (
            match effect {
                Some(x) => quote_expr!(&cx, {
                    $x;
                    $action
                }),
                None    => quote_expr!(&cx, $action),
            },
            use_delayed_transition
        )
    }

    fn create_action_expr(cx: &ExtCtxt, ca_vec: &Vec<CondAction>, states: &Ident) -> (P<Expr>, bool) {
        match ca_vec.len() {
            0 => panic!("Empty CondAction vector"),
            1 => {
                let ca = &ca_vec[0];
                if ca.guard.is_some() {
                    panic!("CondAction vector with a single CondAction with a condition")
                } else {
                    Self::get_condaction_expr(cx, ca, states)
                }
            },
            2 => {
                let (ca_test, ca_else) =
                    if ca_vec[0].guard.is_some() && ca_vec[1].guard.is_some() {
                        if ca_vec[0].guard.as_ref().unwrap() == "else" {
                            (&ca_vec[1],&ca_vec[0])
                        } else {
                            if ca_vec[1].guard.as_ref().unwrap() == "else" {
                                (&ca_vec[0],&ca_vec[1])
                            } else { panic!("Condaction vector with 2 condactions, but neither condition is else") }
                        }
                    } else { panic!("Condaction vector with 2 condactions, but both dont have guards") };
                let test_guard  = str_to_ident(ca_test.guard.as_ref().unwrap());
                let (test_expr, test_use_delayed_transition) = Self::get_condaction_expr(cx, ca_test, states);
                let (else_expr, else_use_delayed_transition) = Self::get_condaction_expr(cx, ca_else, states);
                assert_eq!(test_use_delayed_transition, else_use_delayed_transition);
                (
                    quote_expr!(&cx, {
                        if $test_guard {
                            $test_expr
                        } else {
                            $else_expr
                        }
                    }),
                    test_use_delayed_transition
                )
            },
            _ => panic!("CondAction vector with more than 2 condactions is not supported")
        }
    }

    fn create_state_impl(&mut self, state: &State, events: &Ident, states: &Ident, shr_dat: &Ident, timeout: &Ident) -> P<Item> {
        let cx = self.extctxt();
        let state_ident = str_to_ident(state.name.as_str());
        let mut arms: Vec<Arm> = Vec::new();
        let mut entry_extra = Vec::new();
        let mut exit_extra = Vec::new();
        for (evt, ca_vec) in state.actions.iter() {
            let pat = match *evt {
                Event::Time {ref name, ref relative, ref timeout_ms, ..} => {
                    let nam = str_to_ident(name);
                    entry_extra.push(
                        quote_expr!(&cx,
                            $timeout::start_timer($timeout::$nam, $timeout_ms);
                        )
                    );
                    exit_extra.push(
                        quote_expr!(&cx,
                            $timeout::stop_timer($timeout::$nam);
                        )
                    );
                    quote_pat!(&cx, hsm::Event::User($events::$timeout($timeout::$nam)))
                },
                Event::Signal {ref name, ..} => {
                    let nam = str_to_ident(name);
                    quote_pat!(&cx, hsm::Event::User($events::$nam))
                },
                _ => continue
            };
            let (expr, use_delayed_transition) = Self::create_action_expr(&cx, ca_vec, states);
            arms.push(cx.arm(DUMMY_SP,
                             vec!(pat),
                             match use_delayed_transition {
                                 true  => quote_expr!(&cx, hsm_delayed_transition!(probe, { $expr })),
                                 false => quote_expr!(&cx, $expr)
                             }
            ));
        };
        let mut ordered_arms = Vec::new();
        Self::create_enter_exit_arm(&cx, &state.entry, "Enter", entry_extra).map(|x| ordered_arms.push(x));
        Self::create_enter_exit_arm(&cx, &state.exit, "Exit", exit_extra).map(|x| ordered_arms.push(x));
        ordered_arms.extend(arms);
        match state.actions.get(&Event::Any) {
            Some(ref ca_vec) => ordered_arms.push(cx.arm(DUMMY_SP,
                             vec!(quote_pat!(&cx, _)),
                             {
                                let (expr, use_delayed_transition) = Self::create_action_expr(&cx, ca_vec, states);
                                match use_delayed_transition {
                                    true  => quote_expr!(&cx, hsm_delayed_transition!(probe, { $expr })),
                                    false => quote_expr!(&cx, $expr)
                                }
                             }
            )),
            None => ordered_arms.push(cx.arm(DUMMY_SP,
                             vec!(quote_pat!(&cx, _)),
                             quote_expr!(&cx, hsm::Action::Parent)
            ))
        }
        let match_expr = cx.expr_match(DUMMY_SP, quote_expr!(&cx, evt), ordered_arms);
        quote_item!(&cx,
            impl hsm::State<$events, $states, $shr_dat> for $state_ident {
                fn handle_event(&mut self, shr: &mut $shr_dat, evt: hsm::Event<$events>, probe: bool) -> hsm::Action<$states> {
                    $match_expr
                }
            }
        ).unwrap()
    }

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
