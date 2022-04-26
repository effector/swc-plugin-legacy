use std::cell::RefCell;
use std::rc::Rc;

use swc_atoms::JsWord;
use swc_common::collections::AHashMap;
use swc_ecmascript::visit::{Visit, VisitWith};
use swc_ecmascript::{
    ast::*,
    utils::{ident::IdentLike, Id},
    visit::{as_folder, noop_visit_mut_type, noop_visit_type, Fold, VisitMut},
};

use crate::Config;

#[derive(Debug, Default)]
pub struct State {
    // TODO: do we need to add support for require?
    imported_local_name: Option<Id>,

    // Namespace imports
    imported_local_ns: Option<Id>,
    import_name_cache: RefCell<AHashMap<Id, Id>>,
}

pub fn analyzer(config: Rc<Config>, state: Rc<RefCell<State>>) -> impl VisitMut + Fold {
    as_folder(AsAnalyzer { config, state })
}

struct AsAnalyzer {
    config: Rc<Config>,
    state: Rc<RefCell<State>>,
}

impl VisitMut for AsAnalyzer {
    noop_visit_mut_type!();

    fn visit_mut_module(&mut self, p: &mut Module) {
        let mut v = Analyzer {
            config: &self.config,
            state: &mut *self.state.borrow_mut(),
        };

        p.visit_with(&mut v);
    }

    fn visit_mut_script(&mut self, p: &mut Script) {
        let mut v = Analyzer {
            config: &self.config,
            state: &mut *self.state.borrow_mut(),
        };

        p.visit_with(&mut v);
    }
}

pub fn analyze(config: &Config, program: &Program) -> State {
    let mut state = State::default();

    let mut v = Analyzer {
        config,
        state: &mut state,
    };

    program.visit_with(&mut v);

    state
}

struct Analyzer<'a> {
    config: &'a Config,
    state: &'a mut State,
}

impl Visit for Analyzer<'_> {
    noop_visit_type!();

    fn visit_import_decl(&mut self, i: &ImportDecl) {
        let from = i.src.value.clone();
        for spec in &i.specifiers {
            match spec {
                ImportSpecifier::Named(s) => {
                    println!(
                        "Found named specifier {} from {}",
                        s.local.to_string(),
                        from
                    );
                }
                ImportSpecifier::Default(_) => {
                    println!("Found default import from {}", from);
                }
                ImportSpecifier::Namespace(ns) => {
                    println!("Found import * as {} from {}", ns.local.to_string(), from);
                }
            }
        }
    }
}
