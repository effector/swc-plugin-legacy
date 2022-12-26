use std::{fs::read_to_string, path::PathBuf};

use effector_swc_plugin::{Config, Effector, InternalConfig, PublicConfig};
use swc_common::{FilePathMapping, Mark, SourceMap};
use swc_core::{
    common::chain,
    ecma::{
        transforms::testing::{test_fixture, FixtureTestConfig},
        visit::{as_folder, Fold},
    },
};
use swc_ecmascript::{
    parser::{EsConfig, Syntax},
    transforms::resolver,
};

fn effector<'a>(config: Config, root: &'a str, filename: &'a str, cm: SourceMap) -> impl Fold + 'a {
    as_folder(Effector::new(config, Some(root), Some(filename), cm))
}

#[testing::fixture("tests/fixtures/**/code.js")]
fn fixture(input: PathBuf) {
    let output = input.with_file_name("output.js");

    let root = String::from(input.parent().unwrap().to_string_lossy());
    let filename = String::from(output.to_string_lossy());

    test_fixture(
        Syntax::Es(EsConfig { jsx: true, ..Default::default() }),
        &|_t| {
            let cm = SourceMap::new(FilePathMapping::empty());

            cm.load_file(&input).unwrap();
            let config = input.with_file_name("config.json");

            let string = read_to_string(config).unwrap();

            let public_config = serde_json::from_str::<PublicConfig>(&string).unwrap();

            let config = Config::new(public_config, InternalConfig::new(false));

            chain!(
                resolver(Mark::new(), Mark::new(), false),
                effector(config, &root, &filename, cm)
            )
        },
        &input,
        &output,
        FixtureTestConfig::default(),
    )
}

#[test]
fn test() {}
