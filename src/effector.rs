use std::{
    cell::RefCell,
    collections::hash_map::Entry,
    fmt::{Debug, Formatter},
    path::PathBuf,
    rc::Rc,
    sync::atomic::{AtomicUsize, Ordering},
};

use ahash::{AHashMap, AHashSet};
use swc_core::{
    common::{sync::Lrc, Loc, SourceMapper, DUMMY_SP},
    ecma::{
        ast::*,
        utils::{private_ident, quote_ident},
        visit::{noop_visit_mut_type, VisitMut, VisitMutWith},
    },
};

use crate::{
    config::Config,
    obj_lit,
    path::{normalize_path, strip_root},
    sid::generate_stable_id,
    PublicConfig,
};

macro_rules! enclose {
    ( ($( $x:ident ),*) $y:expr ) => {
        {
            $(let $x = $x.clone();)*
            $y
        }
    };
}

type MethodParserFun = Box<dyn Fn(&State, &Option<&str>)>;

struct MethodParser {
    flag: bool,
    set: AHashSet<String>,
    fun: MethodParserFun,
}

impl MethodParser {
    pub fn new<F: Fn(&State, &Option<&str>) + 'static>(
        flag: bool,
        set: AHashSet<String>,
        fun: F,
    ) -> Self {
        Self { flag, set, fun: Box::new(fun) }
    }
}

impl Debug for MethodParser {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MethodParser").field("flag", &self.flag).field("set", &self.set).finish()
    }
}

type MethodParsers = Vec<MethodParser>;

#[derive(Debug, Default)]
struct UidGenerator {
    filename_count: AtomicUsize,
    factory_count: AtomicUsize,
}

impl UidGenerator {
    fn filename_generate_identifier(&self) -> Ident {
        let old = self.filename_count.fetch_add(1, Ordering::Relaxed);
        private_ident!(format!("_effectorFileName${old}"))
    }

    fn factory_generate_identifier(&self, method: &str) -> Ident {
        let old = self.factory_count.fetch_add(1, Ordering::Relaxed);
        private_ident!(format!("_{method}${old}"))
    }
}

#[derive(Debug, Clone)]
struct SmallConfig {
    add_loc: bool,
    add_names: bool,
    debug_sids: bool,
}

impl From<&PublicConfig> for SmallConfig {
    fn from(p: &PublicConfig) -> Self {
        Self { add_loc: p.add_loc, add_names: p.add_names, debug_sids: p.debug_sids }
    }
}

fn property(key: &str, value: Expr) -> PropOrSpread {
    PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
        key: PropName::Ident(quote_ident!(key)),
        value: Box::new(value),
    })))
}

fn make_trace(
    filename_ident: &Option<Ident>,
    line_number: Option<usize>,
    column_number: Option<usize>,
    uid_generator: &UidGenerator,
) -> Expr {
    let file_line_literal = Expr::from(line_number.unwrap_or(0));

    let file_column_literal = Expr::from(column_number.unwrap_or(0));

    let file_prop = property(
        "file",
        Expr::Ident(
            filename_ident.clone().unwrap_or_else(|| uid_generator.filename_generate_identifier()),
        ),
    );
    let line_prop = property("line", file_line_literal);
    let column_prop = property("column", file_column_literal);

    Expr::Object(ObjectLit { span: DUMMY_SP, props: vec![file_prop, line_prop, column_prop] })
}

fn state_gen_stable_id(state: &State<'_>, name_node_id: &Option<&str>, debug_sids: bool) -> String {
    generate_stable_id(
        state.root.unwrap_or(""),
        state.filename.unwrap_or(""),
        name_node_id,
        state.loc.as_ref().unwrap().line as u32,
        state.loc.as_ref().unwrap().col_display as u32,
        debug_sids,
    )
}

fn set_restore_name_after(
    state: &State<'_>,
    name_node_id: &Option<&str>,
    small_config: &SmallConfig,
) {
    let &SmallConfig { add_loc, add_names, debug_sids } = small_config;

    let stable_id = state_gen_stable_id(state, name_node_id, debug_sids);

    let mut args = state.args.borrow_mut();

    let (first_arg, second_arg, old_config) = (args.get(0), args.get(1), args.get(2));

    if first_arg.is_none() || second_arg.is_none() {
        return;
    }

    let mut config_expr = obj_lit!({ "sid": stable_id });

    if let Some(old_config) = old_config {
        config_expr.props.push(property("and", *old_config.expr.clone()));
    }

    if add_loc {
        let loc = state.loc.as_ref();
        let line = loc.map(|l| l.line);
        let column = loc.map(|l| l.col_display);
        let loc_prop = property(
            "loc",
            make_trace(&state.file_name_identifier, line, column, &state.uid_generator),
        );

        config_expr.props.push(loc_prop);
    }

    if let Some(display_name) = name_node_id {
        if add_names {
            config_expr.props.push(property("name", Expr::from(*display_name)))
        }
    }

    let arg = ExprOrSpread::from(Expr::Object(config_expr));

    if old_config.is_some() {
        args[2] = arg;
    } else {
        args.push(arg);
    }
}

fn set_config_for_conf_method(
    state: &State<'_>,
    name_node_id: &Option<&str>,
    small_config: &SmallConfig,
    single_arg: bool,
    allow_empty_args: bool,
) {
    let &SmallConfig { add_loc, add_names, debug_sids } = small_config;

    let stable_id = state_gen_stable_id(state, name_node_id, debug_sids);

    let mut args = state.args.borrow_mut();

    if args.get(0).is_none() && !allow_empty_args {
        return;
    }

    let common_args = if single_arg {
        args.remove(0)
    } else {
        ExprOrSpread::from(Expr::Array(ArrayLit {
            span: DUMMY_SP,
            elems: args.drain(..).map(Some).collect(),
        }))
    };

    let mut config_expr = obj_lit!({ "sid": stable_id });

    if add_loc {
        let loc = state.loc.as_ref();
        let line = loc.map(|l| l.line);
        let column = loc.map(|l| l.col_display);
        let loc_prop = property(
            "loc",
            make_trace(&state.file_name_identifier, line, column, &state.uid_generator),
        );

        config_expr.props.push(loc_prop);
    }

    if let Some(display_name) = name_node_id {
        if add_names {
            config_expr.props.push(property("name", Expr::from(*display_name)))
        }
    }

    let arg = ExprOrSpread::from(swc_core::quote!(
        "{and: $common, or: $config}" as Expr,
        common: Expr = *common_args.expr,
        config: Expr = config_expr.into()
    ));

    args.insert(0, arg);
}

fn set_event_name_after(
    state: &State<'_>,
    name_node_id: &Option<&str>,
    small_config: &SmallConfig,
) {
    let &SmallConfig { add_loc, add_names, debug_sids } = small_config;

    let stable_id = state_gen_stable_id(state, name_node_id, debug_sids);

    let mut args = state.args.borrow_mut();

    let first_argument = args.get(0);
    let first_argument_exists = first_argument.is_some();

    if first_argument.is_none() {
        if let Some(display_name) = name_node_id {
            args.insert(0, ExprOrSpread::from(Expr::from(*display_name)))
        }
    }

    let old_config = args.get(1);

    let mut config_expr = obj_lit!({ "sid": stable_id });

    if let Some(old_config) = old_config {
        config_expr.props.push(property("and", *old_config.expr.clone()));
    }

    if add_loc {
        let loc = state.loc.as_ref();
        let line = loc.map(|l| l.line);
        let column = loc.map(|l| l.col_display);
        let loc_prop = property(
            "loc",
            make_trace(&state.file_name_identifier, line, column, &state.uid_generator),
        );

        config_expr.props.push(loc_prop);
    }

    if let Some(display_name) = name_node_id {
        if add_names {
            config_expr.props.push(property("name", Expr::from(*display_name)))
        }
    }

    let arg = ExprOrSpread::from(Expr::Object(config_expr));

    if first_argument_exists {
        args.insert(1, arg);
    } else {
        let old_arg = args.get_mut(0);

        if let Some(old_arg) = old_arg {
            *old_arg = arg;
        } else {
            args.insert(0, arg);
        }
    }
}

fn set_store_name_after(
    state: &State<'_>,
    name_node_id: &Option<&str>,
    small_config: &SmallConfig,
    _fill_first_arg: bool,
) {
    let &SmallConfig { add_loc, add_names, debug_sids } = small_config;

    let stable_id = state_gen_stable_id(state, name_node_id, debug_sids);

    let mut args = state.args.borrow_mut();
    let old_config = args.get(1);

    let mut config_expr = obj_lit!({ "sid": stable_id });

    if let Some(old_config) = old_config {
        config_expr.props.push(property("and", *old_config.expr.clone()));
    };

    if add_loc {
        let loc = state.loc.as_ref();
        let line = loc.map(|l| l.line);
        let column = loc.map(|l| l.col_display);
        let loc_prop = property(
            "loc",
            make_trace(&state.file_name_identifier, line, column, &state.uid_generator),
        );

        config_expr.props.push(loc_prop);
    }

    if let Some(display_name) = name_node_id {
        if add_names {
            config_expr.props.push(property("name", Expr::from(*display_name)))
        }
    }

    let arg = ExprOrSpread { expr: Box::new(Expr::Object(config_expr)), spread: None };

    if old_config.is_some() {
        args[1] = arg;
    } else {
        args.push(arg);
    }
}

fn apply_method_parsers(
    method_parsers: &MethodParsers,
    state: &State,
    local: &str,
    resolved: &str,
    id: &Option<&str>,
) {
    for method_parser in method_parsers {
        let MethodParser { fun, flag, set } = method_parser;

        if *flag && set.contains(resolved) {
            if state.is_local_ident(local) {
                return;
            }

            fun(state, &if let Some(id) = id { Some(id) } else { Some("inline_unit") })
        }
    }
}

#[derive(Debug, Default)]
pub struct State<'a> {
    file_name_identifier: Option<Ident>,
    root: Option<&'a str>,
    filename: Option<&'a str>,
    args: RefCell<Vec<ExprOrSpread>>,
    loc: Option<Loc>,
    resolved_methods: AHashMap<Id, AHashSet<Id>>,
    decls_visited: AHashSet<Decl>,
    method_parsers: MethodParsers,
    domain_method_parsers: MethodParsers,
    react_method_parsers: MethodParsers,
    uid_generator: UidGenerator,
}

impl<'a> State<'a> {
    fn is_local_ident(&self, id: &str) -> bool {
        self.decls_visited.iter().any(|d| match d {
            Decl::Var(v) => v.decls.iter().any(|d| match &d.name {
                Pat::Ident(ident) => ident.sym.as_ref() == id,
                _ => false,
            }),
            Decl::Fn(f) => f.ident.sym.as_ref() == id,
            _ => false,
        })
    }

    pub fn new(config: Config, root: Option<&'a str>, filename: Option<&'a str>) -> Self {
        let public_rc = Rc::new(config.public);

        let method_parsers = vec![
            MethodParser::new(
                config.internal.stores,
                config.internal.store_creators,
                enclose! { (public_rc) move |state, id| {
                    set_store_name_after(
                        state, id, &SmallConfig::from(public_rc.as_ref()), false
                    )
                }},
            ),
            MethodParser::new(
                config.internal.events,
                config.internal.event_creators,
                enclose! { (public_rc) move |state, id| {
                    set_event_name_after(state, id, &SmallConfig::from(public_rc.as_ref()))
                }},
            ),
            MethodParser::new(
                config.internal.effects,
                config.internal.effect_creators,
                enclose! { (public_rc) move |state, id| {
                    set_event_name_after(state, id, &SmallConfig::from(public_rc.as_ref()))
                }},
            ),
            MethodParser::new(
                config.internal.domains,
                config.internal.domain_creators,
                enclose! { (public_rc) move |state, id| {
                    set_event_name_after(state, id, &SmallConfig::from(public_rc.as_ref()))
                }},
            ),
            MethodParser::new(
                config.internal.restores,
                config.internal.restore_creators,
                enclose! { (public_rc) move |state, id| {
                    set_restore_name_after(state, id, &SmallConfig::from(public_rc.as_ref()))
                }},
            ),
            MethodParser::new(
                config.internal.combines,
                config.internal.combine_creators,
                enclose! { (public_rc ) move |state, id| {
                    set_config_for_conf_method(
                        state, id, &SmallConfig::from(public_rc.as_ref()), false, false
                    )
                }},
            ),
            MethodParser::new(
                config.internal.samples,
                config.internal.sample_creators,
                enclose! { (public_rc ) move |state, id| {
                    set_config_for_conf_method(
                        state, id, &SmallConfig::from(public_rc.as_ref()), false, false
                    )
                }},
            ),
            MethodParser::new(
                config.internal.forwards,
                config.internal.forward_creators,
                enclose! { (public_rc ) move |state, id| {
                    set_config_for_conf_method(
                        state, id, &SmallConfig::from(public_rc.as_ref()), true, false
                    )
                }},
            ),
            MethodParser::new(
                config.internal.guards,
                config.internal.guard_creators,
                enclose! { (public_rc ) move |state, id| {
                    set_config_for_conf_method(
                        state, id, &SmallConfig::from(public_rc.as_ref()), false, false
                    )
                }},
            ),
            MethodParser::new(
                config.internal.attaches,
                config.internal.attach_creators,
                enclose! { (public_rc ) move |state, id| {
                    set_config_for_conf_method(
                        state, id, &SmallConfig::from(public_rc.as_ref()), true, false
                    )
                }},
            ),
            MethodParser::new(
                config.internal.splits,
                config.internal.split_creators,
                enclose! { (public_rc ) move |state, id| {
                    set_config_for_conf_method(
                        state, id, &SmallConfig::from(public_rc.as_ref()), false, false
                    )
                }},
            ),
            MethodParser::new(
                config.internal.apis,
                config.internal.api_creators,
                enclose! { (public_rc ) move |state, id| {
                    set_config_for_conf_method(
                        state, id, &SmallConfig::from(public_rc.as_ref()), false, false
                    )
                }},
            ),
            MethodParser::new(
                config.internal.merges,
                config.internal.merge_creators,
                enclose! { (public_rc) move |state, id| {
                    set_store_name_after(
                        state, id, &SmallConfig::from(public_rc.as_ref()), false
                    )
                }},
            ),
        ];

        let domain_method_parsers = vec![
            MethodParser::new(
                config.internal.stores,
                config.internal.domain_methods.store,
                enclose! { (public_rc) move |state, id| {
                    set_store_name_after(
                        state, id, &SmallConfig::from(public_rc.as_ref()), false
                    )
                }},
            ),
            MethodParser::new(
                config.internal.events,
                config.internal.domain_methods.event,
                enclose! { (public_rc) move |state, id| {
                    set_event_name_after(state, id, &SmallConfig::from(public_rc.as_ref()))
                }},
            ),
            MethodParser::new(
                config.internal.effects,
                config.internal.domain_methods.effect,
                enclose! { (public_rc) move |state, id| {
                    set_event_name_after(state, id, &SmallConfig::from(public_rc.as_ref()))
                }},
            ),
            MethodParser::new(
                config.internal.domains,
                config.internal.domain_methods.domain,
                enclose! { (public_rc) move |state, id| {
                    set_event_name_after(state, id, &SmallConfig::from(public_rc.as_ref()))
                }},
            ),
        ];

        let react_method_parsers = vec![MethodParser::new(
            config.internal.gates,
            config.internal.react_methods.create_gate,
            enclose! { (public_rc) move |state, id| {
                set_config_for_conf_method(
                    state,
                    id,
                    &SmallConfig::from(public_rc.as_ref()),
                    false,
                    true
                )
            }},
        )];

        Self {
            file_name_identifier: None,
            root,
            filename,
            method_parsers,
            decls_visited: AHashSet::new(),
            loc: None,
            args: RefCell::new(vec![]),
            resolved_methods: AHashMap::new(),
            domain_method_parsers,
            react_method_parsers,
            uid_generator: UidGenerator::default(),
        }
    }
}

#[derive(Debug)]
struct FactoryInfo {
    imported_name: String,
}

pub struct Effector<'a, C: SourceMapper> {
    config: Config,
    state: State<'a>,
    ignored_imports: AHashSet<String>,
    candidate_name: Option<Ident>,
    factory_paths: AHashSet<String>,
    factory_map: AHashMap<Id, FactoryInfo>,
    is_factory: AHashSet<Id>,
    need_factory_import: bool,
    factory_import_added: bool,
    imports_to_add: AHashSet<ImportDecl>,
    with_factory_name: Option<Ident>,
    cm: Lrc<C>,
}

impl<'a, C: SourceMapper> Effector<'a, C> {
    pub fn new(
        config: Config,
        root: Option<&'a str>,
        filename: Option<&'a str>,
        cm: C,
    ) -> Effector<'a, C> {
        Self {
            config: config.clone(),
            state: State::new(config, root, filename),
            ignored_imports: AHashSet::new(),
            candidate_name: None,
            factory_paths: AHashSet::new(),
            factory_map: AHashMap::new(),
            need_factory_import: false,
            imports_to_add: AHashSet::new(),
            with_factory_name: None,
            is_factory: AHashSet::new(),
            factory_import_added: false,
            cm: Lrc::new(cm),
        }
    }

    fn add_import(&mut self, method: Ident) -> Ident {
        let local_ident = self.state.uid_generator.factory_generate_identifier(method.as_ref());
        let decl = ImportDecl {
            span: DUMMY_SP,
            src: Box::new(Str::from("effector")),
            type_only: false,
            asserts: None,
            specifiers: vec![ImportSpecifier::Named(ImportNamedSpecifier {
                span: DUMMY_SP,
                local: local_ident.clone(),
                imported: Some(ModuleExportName::Ident(method)),
                is_type_only: false,
            })],
        };

        self.imports_to_add.insert(decl);

        local_ident
    }

    fn add_file_name_identifier(&mut self) -> Option<String> {
        let mut filename_str = None;
        if self.config.public.add_loc && self.state.file_name_identifier.is_none() {
            filename_str = if self.state.filename.is_some() {
                Some(strip_root(
                    self.state.root.unwrap_or(""),
                    self.state.filename.unwrap_or(""),
                    false,
                ))
            } else {
                None
            };

            let filename_ident = self.state.uid_generator.filename_generate_identifier();

            let _ = self.state.file_name_identifier.insert(filename_ident);
        }

        filename_str
    }
}

impl<'a, C: SourceMapper> VisitMut for Effector<'a, C> {
    noop_visit_mut_type!();

    fn visit_mut_decl(&mut self, d: &mut Decl) {
        self.state.decls_visited.insert(d.clone());

        d.visit_mut_children_with(self);
    }

    fn visit_mut_member_prop(&mut self, p: &mut MemberProp) {
        self.candidate_name = if let MemberProp::Ident(id) = p { Some(id.clone()) } else { None };

        p.visit_mut_children_with(self);
    }

    fn visit_mut_module(&mut self, m: &mut Module) {
        let filename = self.add_file_name_identifier();

        if let Some(file_name_ident) = &self.state.file_name_identifier {
            if self.config.public.add_loc {
                let last_import_index = m
                    .body
                    .iter()
                    .rposition(|m| matches!(m, ModuleItem::ModuleDecl(ModuleDecl::Import(_))));
                let stmt = ModuleItem::Stmt(Stmt::Decl(Decl::Var(Box::new(VarDecl {
                    span: DUMMY_SP,
                    kind: VarDeclKind::Var,
                    declare: false,
                    decls: vec![VarDeclarator {
                        span: DUMMY_SP,
                        name: Pat::Ident(BindingIdent::from(file_name_ident.clone())),
                        init: Some(Box::new(Expr::from(filename.unwrap_or_else(|| "".into())))),
                        definite: false,
                    }],
                }))));
                if let Some(index) = last_import_index {
                    m.body.insert(index + 1, stmt);
                } else {
                    m.body.insert(0, stmt);
                };
            }
        }

        m.visit_mut_children_with(self);

        if !self.imports_to_add.is_empty() {
            let last_import_index = m
                .body
                .iter()
                .rposition(|i| matches!(i, ModuleItem::ModuleDecl(ModuleDecl::Import(_))));

            if let Some(mut index) = last_import_index {
                index += 1;
                m.body.splice(
                    index..index,
                    self.imports_to_add
                        .iter()
                        .map(|i| ModuleItem::ModuleDecl(ModuleDecl::Import(i.clone()))),
                );
            }
        }
    }

    fn visit_mut_import_decl(&mut self, d: &mut ImportDecl) {
        let source = &d.src.value.to_string();
        let factories_used = !self.config.public.factories.is_empty();
        let has_relative_factories = self
            .config
            .public
            .factories
            .iter()
            .any(|f| f.starts_with("./") || f.starts_with("../"));

        if self.config.public.import_names.contains(&d.src.value.to_string()) {
            for specifier in &d.specifiers {
                if let ImportSpecifier::Named(named) = specifier {
                    let local = named.local.clone();

                    let key = match &named.imported {
                        Some(ModuleExportName::Ident(ident)) => ident.clone(),
                        _ => local,
                    }
                    .to_id();

                    match self.state.resolved_methods.entry(key) {
                        Entry::Occupied(mut e) => {
                            let locals = e.get_mut();
                            locals.insert(named.local.to_id());
                        }
                        Entry::Vacant(v) => {
                            v.insert(AHashSet::from([named.local.to_id()]));
                        }
                    }
                }
            }
        } else {
            for specifier in &d.specifiers {
                if let ImportSpecifier::Named(named) = specifier {
                    let local = named.local.sym.as_ref();

                    if self.state.method_parsers.iter().map(|m| &m.set).any(|s| s.contains(local)) {
                        self.ignored_imports.insert(local.to_owned());
                    }
                }
            }
        }

        if let Some(bindings) = &self.config.public.bindings {
            let mut check_and_replace =
                |replace: bool, no_scope: &AHashSet<String>, scope: &AHashSet<String>| {
                    if replace && no_scope.contains(source) {
                        d.src = Box::new(
                            scope.iter().find(|s| s.contains("scope")).unwrap().to_string().into(),
                        )
                    }
                };

            if let Some(react) = &bindings.react {
                let import_react_names = &self.config.internal.import_react_names;
                check_and_replace(
                    react.scope_replace,
                    &import_react_names.no_scope,
                    &import_react_names.scope,
                );
            }

            if let Some(solid) = &bindings.solid {
                let import_solid_names = &self.config.internal.import_solid_names;
                check_and_replace(
                    solid.scope_replace,
                    &import_solid_names.no_scope,
                    &import_solid_names.scope,
                );
            }
        }

        if factories_used {
            let root = self.state.root.unwrap_or("");

            if self.factory_paths.is_empty() {
                if has_relative_factories {
                    self.factory_paths = self
                        .config
                        .public
                        .factories
                        .iter()
                        .map(|fab| {
                            if fab.starts_with("./") || fab.starts_with("../") {
                                let resolved_fab =
                                    normalize_path(&PathBuf::from(format!("{root}/{fab}")));
                                strip_root(
                                    root,
                                    &String::from(resolved_fab.to_string_lossy()),
                                    true,
                                )
                            } else {
                                fab.clone()
                            }
                        })
                        .collect()
                } else {
                    self.factory_paths = self.config.public.factories.clone();
                }
            }

            let mut normalized_source: PathBuf = PathBuf::from(d.src.value.to_string());

            if normalized_source.starts_with(".") {
                let current_file = self.state.filename.unwrap_or("");

                let path = PathBuf::from(current_file);

                let dir = path.parent().expect("Should have parent directory");
                let resolved_import = normalize_path(&PathBuf::from(format!(
                    "{dir}/{normalized_source}",
                    dir = dir.display(),
                    normalized_source = normalized_source.display()
                )));

                normalized_source = PathBuf::from(strip_root(
                    root,
                    &String::from(resolved_import.to_string_lossy()),
                    true,
                ));
            }

            normalized_source.set_extension("");

            if self.factory_paths.contains(&String::from(normalized_source.to_string_lossy())) {
                self.need_factory_import = true;

                for specifier in &d.specifiers {
                    let (local_name, imported_name) = match specifier {
                        ImportSpecifier::Default(d) => (d.local.clone(), "default".to_string()),
                        ImportSpecifier::Named(named) => (
                            named.local.clone(),
                            match &named.imported {
                                Some(exported) => match exported {
                                    ModuleExportName::Ident(id) => id.to_string(),
                                    ModuleExportName::Str(str) => str.value.to_string(),
                                },
                                None => named.local.to_string(),
                            },
                        ),
                        _ => continue,
                    };

                    self.factory_map.insert(local_name.to_id(), FactoryInfo { imported_name });
                }
            }
        }

        d.visit_mut_children_with(self);
    }

    fn visit_mut_key_value_prop(&mut self, p: &mut KeyValueProp) {
        self.candidate_name = match &p.key {
            PropName::Ident(id) => Some(id.clone()),
            _ => None,
        };

        p.visit_mut_children_with(self);
    }

    fn visit_mut_var_declarator(&mut self, d: &mut VarDeclarator) {
        let ident = match &d.name {
            Pat::Ident(ident) => Some(ident.id.clone()),
            _ => None,
        };

        self.candidate_name = ident;
        d.visit_mut_children_with(self);
    }

    fn visit_mut_assign_expr(&mut self, e: &mut AssignExpr) {
        let ident = match &e.left {
            PatOrExpr::Pat(p) => match &**p {
                Pat::Ident(i) => Some(i.id.clone()),
                _ => None,
            },
            PatOrExpr::Expr(e) => match &**e {
                Expr::Ident(i) => Some(i.clone()),
                _ => None,
            },
        };

        self.candidate_name = ident;
        e.visit_mut_children_with(self);
    }

    fn visit_mut_call_expr(&mut self, e: &mut CallExpr) {
        let factories_used = !self.config.public.factories.is_empty();

        if let Callee::Expr(expr) = &mut e.callee {
            match &mut **expr {
                Expr::Member(member) => {
                    if let MemberProp::Ident(ident) = &member.prop {
                        let local = ident.sym.to_string();
                        self.state.args = RefCell::new(e.args.clone());

                        let loc = self.cm.lookup_char_pos(ident.span.lo);
                        self.state.loc = Some(loc);
                        
                        apply_method_parsers(
                            &self.state.domain_method_parsers,
                            &self.state,
                            "",
                            &local,
                            &self.candidate_name.as_ref().map(|i| i.as_ref()),
                        );
                    }
                }
                Expr::Ident(ident) => {
                    let local = ident.sym.to_string();

                    if !self.ignored_imports.contains(&local) {
                        let resolved =
                            self.state.resolved_methods.iter().find_map(|(k, locals)| {
                                locals.contains(&ident.to_id()).then_some(k)
                            });
                        if let Some(resolved) = resolved {
                            let loc = self.cm.lookup_char_pos(ident.span.lo);

                            self.state.loc = Some(loc);
                            self.state.args = RefCell::new(e.args.clone());
                            apply_method_parsers(
                                &self.state.method_parsers,
                                &self.state,
                                &local,
                                resolved.0.as_ref(),
                                &self.candidate_name.as_ref().map(|i| i.as_ref()),
                            );
                            apply_method_parsers(
                                &self.state.react_method_parsers,
                                &self.state,
                                &local,
                                resolved.0.as_ref(),
                                &self.candidate_name.as_ref().map(|i| i.as_ref()),
                            );
                        }
                    } else {
                        return;
                    }

                    if self.state.resolved_methods.is_empty() {
                        let loc = self.cm.lookup_char_pos(ident.span.lo);

                        self.state.loc = Some(loc);
                        self.state.args = RefCell::new(e.args.clone());
                        apply_method_parsers(
                            &self.state.method_parsers,
                            &self.state,
                            &local,
                            &local,
                            &self.candidate_name.as_ref().map(|i| i.as_ref()),
                        );
                        apply_method_parsers(
                            &self.state.react_method_parsers,
                            &self.state,
                            &local,
                            &local,
                            &self.candidate_name.as_ref().map(|i| i.as_ref()),
                        );
                    }

                    if factories_used
                        && !self.is_factory.contains(&ident.to_id())
                        && self.factory_map.contains_key(&ident.to_id())
                    {
                        let loc = self.cm.lookup_char_pos(ident.span.lo);

                        self.state.loc = Some(loc.clone());
                        self.state.args = RefCell::new(e.args.clone());
                        if !self.factory_import_added {
                            self.factory_import_added = true;
                            self.with_factory_name =
                                Some(self.add_import(quote_ident!("withFactory")));
                        }

                        let FactoryInfo { imported_name } = self
                            .factory_map
                            .remove(&ident.to_id())
                            .expect("Already checked for existence.");
                        self.is_factory.insert(ident.to_id());

                        let sid = generate_stable_id(
                            self.state.root.unwrap_or(""),
                            self.state.filename.unwrap_or(""),
                            &self.candidate_name.as_ref().map(|i| i.as_ref()),
                            loc.line as u32,
                            loc.col_display as u32,
                            self.config.public.debug_sids,
                        );

                        let expr = swc_core::quote!(
                            "$factory({sid: $sid,fn:()=>$fun})" as Expr,
                            factory = self.with_factory_name.clone().unwrap(),
                            sid: Expr = sid.into(),
                            fun: Expr = Expr::Call(e.clone()),
                        );

                        if let Expr::Call(mut call) = expr {
                            if let Some(arg) = call.args.get_mut(0) {
                                if let Expr::Object(obj) = &mut *arg.expr {
                                    if self.config.public.add_loc || self.config.public.add_names {
                                        let name_prop = property(
                                            "name",
                                            Expr::from(
                                                self.candidate_name
                                                    .clone()
                                                    .map(|n| n.sym)
                                                    .unwrap_or_else(|| "inline_unit".into()),
                                            ),
                                        );
                                        let method_prop =
                                            property("method", Expr::from(imported_name));
                                        obj.props.extend([name_prop, method_prop]);
                                    }

                                    if self.config.public.add_loc {
                                        let loc_prop = property(
                                            "loc",
                                            make_trace(
                                                &self.state.file_name_identifier,
                                                Some(loc.line),
                                                Some(loc.col_display),
                                                &self.state.uid_generator,
                                            ),
                                        );
                                        obj.props.push(loc_prop);
                                    }
                                }
                            }
                            *e = call;
                            e.visit_mut_children_with(self);
                            return;
                        }
                    }
                }
                _ => (),
            }
        }

        let mut args = self.state.args.borrow_mut();
        if !args.is_empty() {
            e.args = args.drain(..).collect();
        }

        drop(args);

        e.visit_mut_children_with(self);
    }
}
