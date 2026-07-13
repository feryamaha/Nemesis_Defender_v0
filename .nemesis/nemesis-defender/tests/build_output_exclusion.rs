//! Regressao P1 (SPEC_022): exclusao de build-output por COMPONENTE de path.
use nemesis_defender::is_path_excluded;
use std::path::Path;

#[test]
fn next_outputs_are_excluded() {
    assert!(is_path_excluded(Path::new(".next/static/chunks/main-abc123.js")));
    assert!(is_path_excluded(Path::new(".next/server/app/page.js")));
    assert!(is_path_excluded(Path::new(".next/cache/turbopack/0.pack")));
    assert!(is_path_excluded(Path::new("/home/u/proj/.next/static/chunks/x.js")));
}

#[test]
fn other_build_dirs_are_excluded() {
    assert!(is_path_excluded(Path::new("dist/index.js")));
    assert!(is_path_excluded(Path::new("build/static/js/main.js")));
    assert!(is_path_excluded(Path::new("out/index.html")));
}

#[test]
fn source_is_not_excluded() {
    assert!(!is_path_excluded(Path::new("src/index.ts")));
    assert!(!is_path_excluded(Path::new("src/components/App.tsx")));
}

#[test]
fn lookalike_dirs_are_not_excluded() {
    // Component match (nao substring): `myapp.next/` NAO e o build-dir `.next/`.
    // Com o match antigo por substring `.contains(".next/")`, isto seria isentado por engano.
    assert!(!is_path_excluded(Path::new("myapp.next/bundle.js")));
    assert!(!is_path_excluded(Path::new("src/distribute/util.ts")));
}
