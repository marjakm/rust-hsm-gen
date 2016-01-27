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
    Transition { state:        String,          effect: Option<String>},
    Diverge    { cond_act_vec: Vec<CondAction>, effect: Option<String>},
}

impl Action {
    pub fn from_transition(t: &Transition, sm: &HashMap<String, State>, vm: &HashMap<String, Subvertex>) -> Self {
        Self::from_transition_with_effect(t, sm, vm, None)
    }

    fn from_transition_with_effect(t: &Transition, sm: &HashMap<String, State>, vm: &HashMap<String, Subvertex>, effect: Option<String>) -> Self {
        assert!(t.guard.is_none());
        assert!(t.trigger.is_none());

        let eff = match (effect, &t.effect) {
            (Some(ref a), &Some(ref b)) => Some(format!("{{{};{}}}", a, b)),
            (None,        b           ) => b.clone(),
            (a,           &None       ) => a,
        };

        if let Some(state) = sm.get(&t.target_id) {
            match state.initial_transition {
                Some(ref trans) => Self::from_transition_with_effect(trans, sm, vm, eff),
                None            => Action::Transition { state: state.name.clone(), effect: eff }
            }
        } else {
            if let Some(subvertex) = vm.get(&t.target_id) {
                match *subvertex {
                    Subvertex::Initial  {..}                   => panic!("Transition to initial state is forbidden"),
                    Subvertex::Final    {..}                   => panic!("Implement transition to final state"),
                    Subvertex::State    {ref state, ..}        => panic!("CondAction get_target: found state in subvertex map"),
                    Subvertex::Junction {ref transition, ..}   => Self::from_transition_with_effect(&transition, sm, vm, eff),
                    Subvertex::Choice   {ref transitions, ..}  => Action::Diverge {
                        cond_act_vec: transitions.iter().map(|x| CondAction::from_transition((*x).clone(), sm, vm)).collect(),
                        effect:       eff
                    },
                }
            } else {
                panic!("CondAction get_target for {:?}: target_id {} not in subvertex map", t, t.target_id)
            }
        }
    }
}
