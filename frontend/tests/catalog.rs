//! The catalog is the front-end's texts (`backend/translations/<area>`) and
//! the messages each back-end module ships (`backend/src/<module>/messages`).

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn every_key_the_pages_use_is_in_the_polish_catalog() {
    let missing = keys_missing_from("pl.json");

    assert_eq!(missing, Vec::<String>::new());
}

#[test]
fn every_key_the_pages_use_is_in_the_english_catalog() {
    let missing = keys_missing_from("en.json");

    assert_eq!(missing, Vec::<String>::new());
}

#[test]
fn every_key_of_the_catalog_is_used_by_one_of_the_applications() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut used = keys_used_in(&root.join("src"));
    used.extend(keys_used_in(&root.join("../backend/src")));

    let unused: Vec<String> = catalog(root, "en.json")
        .into_keys()
        .filter(|key| !used.contains(key))
        .collect();

    assert_eq!(unused, Vec::<String>::new());
}

#[test]
fn the_scan_finds_the_keys_of_the_pages() {
    let used = keys_used_in(&Path::new(env!("CARGO_MANIFEST_DIR")).join("src"));

    assert!(used.contains(&"layout.nav.files".to_owned()), "{used:?}");
    assert!(used.contains(&"legal.terms.p1".to_owned()), "{used:?}");
    assert!(!used.contains(&"terms.html".to_owned()), "{used:?}");
}

/// Every message of one language, `pl.json` or `en.json`, from every file
/// that ships one.
fn catalog(root: &Path, file: &str) -> BTreeMap<String, String> {
    let backend = root.join("../backend");
    let mut folders = subfolders(&backend.join("translations"));
    folders.extend(
        subfolders(&backend.join("src"))
            .into_iter()
            .map(|module| module.join("messages"))
            .filter(|messages| messages.is_dir()),
    );
    folders
        .iter()
        .flat_map(|folder| {
            let text = fs::read_to_string(folder.join(file)).expect("each folder ships both languages");
            serde_json::from_str::<BTreeMap<String, String>>(&text).expect("a catalog is a JSON object")
        })
        .collect()
}

fn subfolders(dir: &Path) -> Vec<PathBuf> {
    fs::read_dir(dir)
        .expect("the back-end is next to the front-end")
        .map(|entry| entry.expect("a folder entry is readable").path())
        .filter(|path| path.is_dir())
        .collect()
}

fn keys_missing_from(file: &str) -> Vec<String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let messages = catalog(root, file);
    keys_used_in(&root.join("src"))
        .into_iter()
        .filter(|key| !messages.contains_key(key))
        .collect()
}

/// The string literals outside tests that look like message keys: dotted
/// lower-case words, such as `files.list.title`.
fn keys_used_in(dir: &Path) -> Vec<String> {
    let mut keys = Vec::new();
    for path in rust_files(dir) {
        let source = fs::read_to_string(&path).expect("a source file is readable");
        let code = source.split("#[cfg(test)]").next().unwrap_or_default();
        keys.extend(
            code.split('"')
                .skip(1)
                .step_by(2)
                .filter(|literal| looks_like_key(literal))
                .map(str::to_owned),
        );
    }
    keys.sort();
    keys.dedup();
    keys
}

fn looks_like_key(literal: &str) -> bool {
    let words: Vec<&str> = literal.split('.').collect();
    words.len() > 1
        && !std::path::Path::new(literal)
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("html"))
        && words.iter().all(|word| {
            word.starts_with(|c: char| c.is_ascii_lowercase())
                && word
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        })
}

fn rust_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir).expect("the source folder is readable") {
        let path = entry.expect("a folder entry is readable").path();
        if path.is_dir() {
            files.extend(rust_files(&path));
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }
    files
}
