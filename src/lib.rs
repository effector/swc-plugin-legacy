use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use serde::Deserialize;
use swc_atoms::JsWord;
use swc_common::{chain, SourceFile};
use swc_ecmascript::visit::{Fold, VisitMut};

mod analyzer;

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Config {
    #[serde(default = "effector_by_default")]
    pub import_name: JsWord,

    #[serde(default = "Vec::new")]
    pub factories: Vec<JsWord>,
}

fn effector_by_default() -> JsWord {
    "effector".into()
}

pub fn effector(file: Arc<SourceFile>, config: Config) -> impl Fold + VisitMut {
    let state: Rc<RefCell<analyzer::State>> = Default::default();
    let config = Rc::new(config);

    analyzer::analyzer(config.clone(), state.clone())
}
