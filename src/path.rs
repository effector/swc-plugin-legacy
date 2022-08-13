use std::path::{Component, Path, PathBuf};

pub fn normalize_path(path: &Path) -> PathBuf {
    let mut components = path.components().peekable();
    let mut ret = if let Some(c @ Component::Prefix(..)) = components.peek().cloned() {
        components.next();
        PathBuf::from(c.as_os_str())
    } else {
        PathBuf::new()
    };

    for component in components {
        match component {
            Component::Prefix(..) => unreachable!(),
            Component::RootDir => {
                ret.push(component.as_os_str());
            }
            Component::CurDir => {}
            Component::ParentDir => {
                ret.pop();
            }
            Component::Normal(c) => {
                ret.push(c);
            }
        }
    }
    ret
}

pub fn strip_root(babel_root: &str, filename: &str, omit_first_slash: bool) -> String {
    let raw = PathBuf::from(filename.replace(babel_root, ""));
    let normalized = normalize_path(&raw).to_str().unwrap().to_string();
    let mut normalized_seq = normalized.split('/').collect::<Vec<_>>();

    let len = normalized_seq.len();
    let slice = &normalized_seq.clone()[1..];
    if omit_first_slash && len > 0 && normalized_seq[0].is_empty() {
        normalized_seq = Vec::from(slice);
    };

    normalized_seq.join("/")
}
