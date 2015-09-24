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
use super::{Transition, Subvertex, CondAction, State};


#[derive(Debug, Clone)]
pub enum Action {
    Ignore,
    Parent,
    Transition(String),
    Diverge(Vec<CondAction>)
}

impl Action {
    pub fn from_transition(t: &Transition, sm: &HashMap<String, State>, vm: &HashMap<String, Subvertex>) -> Self {
        assert!(t.guard.is_none());
        assert!(t.effect.is_none());
        assert!(t.trigger.is_none());

        if let Some(state) = sm.get(&t.target_id) {
            match state.initial_transition {
                Some(ref trans) => Self::from_transition(trans, sm, vm),
                None            => Action::Transition(state.name.clone())
            }
        } else {
            if let Some(subvertex) = vm.get(&t.target_id) {
                match *subvertex {
                    Subvertex::Initial  {..}                   => panic!("Transition to initial state is forbidden"),
                    Subvertex::Final    {..}                   => panic!("Implement transition to final state"),
                    Subvertex::State    {ref state, ..}        => panic!("CondAction get_target: found state in subvertex map"),
                    Subvertex::Junction {ref transition, ..}   => Self::from_transition(&transition, sm, vm),
                    Subvertex::Choice   {ref transitions, ..}  => Action::Diverge(
                        transitions.iter().map(|x| CondAction::from_transition((*x).clone(), sm, vm)).collect()
                    ),
                }
            } else {
                panic!("CondAction get_target: target_id {} not in subvertex map", t.target_id)
            }
        }
    }
}
