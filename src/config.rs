euse std::{fmt, path::PathBuf};

use ahash::AHashSet;
use serde::{
    de,
    de::{value, SeqAccess, Visitor},
    Deserialize, Deserializer,
};

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Binding {
    #[serde(default)]
    pub scope_replace: bool,
    #[serde(default)]
    pub methods: AHashSet<String>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Bindings {
    pub react: Option<Binding>,
    pub solid: Option<Binding>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PublicConfig {
    #[serde(default = "r#true")]
    pub add_names: bool,
    #[serde(default)]
    pub add_loc: bool,
    #[serde(default)]
    pub debug_sids: bool,
    #[serde(default)]
    pub filename: bool,
    #[serde(default)]
    pub no_defaults: bool,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_factories")]
    pub factories: AHashSet<String>,
    #[serde(default = "default_import_names")]
    #[serde(deserialize_with = "deserialize_import_names")]
    pub import_names: AHashSet<String>,
    pub bindings: Option<Bindings>,
}

#[derive(Debug, Clone)]
pub(crate) struct DomainMethods {
    pub(crate) store: AHashSet<String>,
    pub(crate) event: AHashSet<String>,
    pub(crate) effect: AHashSet<String>,
    pub(crate) domain: AHashSet<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct ReactMethods {
    pub(crate) create_gate: AHashSet<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ImportViewNames {
    pub scope: AHashSet<String>,
    pub no_scope: AHashSet<String>,
}

#[derive(Debug, Clone)]
pub struct InternalConfig {
    pub stores: bool,
    pub events: bool,
    pub effects: bool,
    pub domains: bool,
    pub restores: bool,
    pub combines: bool,
    pub samples: bool,
    pub forwards: bool,
    pub guards: bool,
    pub attaches: bool,
    pub splits: bool,
    pub apis: bool,
    pub merges: bool,
    pub gates: bool,
    pub store_creators: AHashSet<String>,
    pub event_creators: AHashSet<String>,
    pub effect_creators: AHashSet<String>,
    pub domain_creators: AHashSet<String>,
    pub restore_creators: AHashSet<String>,
    pub combine_creators: AHashSet<String>,
    pub sample_creators: AHashSet<String>,
    pub forward_creators: AHashSet<String>,
    pub guard_creators: AHashSet<String>,
    pub attach_creators: AHashSet<String>,
    pub split_creators: AHashSet<String>,
    pub api_creators: AHashSet<String>,
    pub merge_creators: AHashSet<String>,
    pub import_react_names: ImportViewNames,
    pub import_solid_names: ImportViewNames,
    pub(crate) domain_methods: DomainMethods,
    pub(crate) react_methods: ReactMethods,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub public: PublicConfig,
    pub internal: InternalConfig,
}

impl Config {
    pub fn new(public: PublicConfig, internal: InternalConfig) -> Self {
        Self { public, internal }
    }
}

impl InternalConfig {
    pub fn new(_no_defaults: bool) -> Self {
        let store_creators = AHashSet::from_iter(vec!["createStore".into()]);
        let event_creators = AHashSet::from_iter(vec!["createEvent".into()]);
        let effect_creators = AHashSet::from_iter(vec!["createEffect".into()]);
        let domain_creators = AHashSet::from_iter(vec!["createDomain".into()]);
        let restore_creators = AHashSet::from_iter(vec!["restore".into()]);
        let combine_creators = AHashSet::from_iter(vec!["combine".into()]);
        let sample_creators = AHashSet::from_iter(vec!["sample".into()]);
        let forward_creators = AHashSet::from_iter(vec!["forward".into()]);
        let guard_creators = AHashSet::from_iter(vec!["guard".into()]);
        let attach_creators = AHashSet::from_iter(vec!["attach".into()]);
        let split_creators = AHashSet::from_iter(vec!["split".into()]);
        let api_creators = AHashSet::from_iter(vec!["createApi".into()]);
        let merge_creators = AHashSet::from_iter(vec!["merge".into()]);
        let domain_methods = DomainMethods {
            store: AHashSet::from_iter(["store".into(), "createStore".into()]),
            event: AHashSet::from_iter(["event".into(), "createEvent".into()]),
            effect: AHashSet::from_iter(["effect".into(), "createEffect".into()]),
            domain: AHashSet::from_iter(["domain".into(), "createDomain".into()]),
        };
        let react_methods =
            ReactMethods { create_gate: AHashSet::from_iter(["createGate".into()]) };

        Self {
            stores: true,
            events: true,
            effects: true,
            domains: true,
            restores: true,
            combines: true,
            samples: true,
            forwards: true,
            guards: true,
            attaches: true,
            splits: true,
            apis: true,
            merges: true,
            gates: true,
            store_creators,
            event_creators,
            effect_creators,
            domain_creators,
            restore_creators,
            combine_creators,
            sample_creators,
            forward_creators,
            guard_creators,
            attach_creators,
            split_creators,
            api_creators,
            merge_creators,
            domain_methods,
            react_methods,
            import_react_names: ImportViewNames {
                scope: AHashSet::from_iter([
                    "effector-react/scope".into(),
                    "effector-react/ssr".into(),
                ]),
                no_scope: AHashSet::from_iter([
                    "effector-react".into(),
                    "effector-react/compat".into(),
                ]),
            },
            import_solid_names: ImportViewNames {
                scope: AHashSet::from_iter(["effector-solid/scope".into()]),
                no_scope: AHashSet::from_iter(["effector-solid".into()]),
            },
        }
    }
}

fn r#true() -> bool {
    true
}

fn default_import_names() -> AHashSet<String> {
    AHashSet::from([
        "effector".into(),
        "effector/compat".into(),
        "effector-root".into(),
        "effector-root/compat".into(),
        "effector-logger".into(),
        "trail/runtime".into(),
        "patronum".into(),
        "@effector/effector".into(),
        "@farfetched/core".into(),
        "@effector/reflect".into(),
        "@effector/reflect/ssr".into(),
        "atomic-router".into()
    ])
}

fn deserialize_import_names<'de, D>(deserializer: D) -> Result<AHashSet<String>, D::Error>
where
    D: Deserializer<'de>,
{
    struct StringOrHashSet;

    impl<'de> Visitor<'de> for StringOrHashSet {
        type Value = AHashSet<String>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string or list of strings")
        }

        fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(AHashSet::from([s.to_owned()]))
        }

        fn visit_seq<S>(self, seq: S) -> Result<Self::Value, S::Error>
        where
            S: SeqAccess<'de>,
        {
            Deserialize::deserialize(value::SeqAccessDeserializer::new(seq))
        }
    }

    deserializer.deserialize_any(StringOrHashSet)
}

fn deserialize_factories<'de, D>(deserializer: D) -> Result<AHashSet<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let res: AHashSet<PathBuf> = AHashSet::deserialize(deserializer)?;

    Ok(res
        .into_iter()
        .map(|mut p| {
            p.set_extension("");
            String::from(p.to_string_lossy())
        })
        .collect())
}
