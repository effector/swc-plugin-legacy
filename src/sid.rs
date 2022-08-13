use std::hash::{BuildHasher, Hash, Hasher};

use ahash::RandomState;
use radix_fmt::radix_36;

use crate::path::strip_root;

pub fn generate_stable_id(
    babel_root: &str,
    filename: &str,
    var_name: &Option<&str>,
    line: u32,
    column: u32,
    debug_sids: bool,
) -> String {
    let state = RandomState::with_seeds(0xD3ADB33F, 0xF00DBABE, 0xCAF3BAB3, 0x8BADF00D);
    let var_name = var_name.unwrap_or("");
    let mut hasher = state.build_hasher();
    let normalized = strip_root(babel_root, filename, false);

    let appendix = if debug_sids { format!(":{normalized}:{var_name}") } else { "".to_string() };

    let res = format!("{var_name} {normalized} [{line}, {column}]");

    res.as_bytes().hash(&mut hasher);

    format!("{}{appendix}", radix_36(hasher.finish()))
}

#[cfg(test)]
mod test {
    use crate::sid::generate_stable_id;

    #[test]
    fn test_hash() {
        // 2irlsmdmp0jlj
        let res = generate_stable_id(
            "/Users/k.mironov/WebstormProjects/test",
            "/Users/k.mironov/WebstormProjects/test/node_modules/.vite/deps/atomic-router-solid.\
             js?v=4bf0bdac",
            &Some(""),
            42,
            109,
            false,
        );

        assert_eq!(res, "2irlsmdmp0jlj")
    }
}
