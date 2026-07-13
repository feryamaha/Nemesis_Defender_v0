//! Regressao SPEC_023: o daemon vigia o diretorio de config do Grok Build.
use nemesis_defender::watcher::WATCH_PATHS;

#[test]
fn grok_dir_is_watched() {
    assert!(
        WATCH_PATHS.contains(&".grok"),
        "WATCH_PATHS deve conter \".grok\" para vigiar a config do Grok Build"
    );
}
