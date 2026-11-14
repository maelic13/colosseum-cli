use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[test]
fn cli_dependency_graph_has_no_windowing_database_or_scheduler_package() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let output = Command::new(env!("CARGO"))
        .args([
            "tree",
            "-p",
            "colosseum-cli",
            "--no-default-features",
            "--target",
            "all",
            "--prefix",
            "none",
            "--format",
            "{p}",
        ])
        .current_dir(root)
        .output()
        .expect("cargo tree");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let tree = String::from_utf8(output.stdout).unwrap();
    let packages = tree
        .lines()
        .filter_map(|line| line.split_whitespace().next())
        .collect::<HashSet<_>>();
    assert!(packages.contains("colosseum-engine"), "{tree}");
    for forbidden in [
        "colosseum-gui",
        "eframe",
        "egui",
        "egui-winit",
        "winit",
        "rusqlite",
        "libsqlite3-sys",
        "crossbeam-channel",
    ] {
        assert!(
            !packages.contains(forbidden),
            "headless CLI depends on forbidden package {forbidden}"
        );
    }
}

#[test]
fn cli_source_has_no_gui_state_or_app_directory_access() {
    let source_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let source = source_files(&source_root)
        .into_iter()
        .map(|path| fs::read_to_string(path).unwrap())
        .collect::<String>();
    for forbidden in [
        "AppDirs",
        "EngineLibrary",
        "ProjectDirs",
        "APPDATA",
        "engines.json",
        "colosseum.sqlite",
    ] {
        assert!(
            !source.contains(forbidden),
            "CLI source contains {forbidden}"
        );
    }
}

#[test]
fn the_release_lane_is_the_only_one_and_is_complete_and_least_privileged() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    // The desktop application releases from its own repository (ADR-0011).
    assert!(!root.join(".github/workflows/release-gui.yml").exists());
    assert!(!root.join(".github/workflows/release-cli.yml").exists());
    let release = fs::read_to_string(root.join(".github/workflows/release.yml")).unwrap();

    // Plain `vX.Y.Z` tags publish (ADR-0012); a candidate is built on every
    // pull request into master and on a manual dispatch, never on a branch
    // push.
    assert!(release.contains("  push:\n    tags:\n      - 'v[0-9]+.[0-9]+.[0-9]+'\n"));
    assert!(!release.contains("'cli-v"));
    assert!(!release.contains("'gui-v"));
    assert!(release.contains("workflow_dispatch:"));
    assert!(release.contains("  pull_request:\n    branches: [master]\n"));
    assert_eq!(release.matches("branches:").count(), 1);
    assert!(
        release.contains("github.event_name == 'push' && startsWith(github.ref, 'refs/tags/v')")
    );
    assert!(!release.contains("origin/main"));

    // The lane builds, packages and smokes the product through the one build
    // entry point.
    assert!(release.contains("cargo xtask package cli"));

    assert!(release.contains("permissions:\n  contents: read"));
    assert_eq!(release.matches("contents: write").count(), 1);
    // A candidate proves the whole lane without a tag or a release.
    assert!(release.contains("CANDIDATE.json"));
    // The published set is an exact list checked by name and by count, so a
    // release carries what its matrix produced and nothing else.
    assert!(release.contains("expected=("));
    assert!(
        release.contains(
            r#"for file in "${expected[@]}"; do test -f "release-artifacts/$file"; done"#
        )
    );
    assert!(release.contains("(cd release-artifacts && sha256sum --check SHA256SUMS)"));
    // Checksums are generated and re-checked, never published as an asset.
    assert!(!release.contains("release-artifacts/SHA256SUMS\n"));
}

#[test]
fn ci_gates_pull_requests_into_master() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let ci = fs::read_to_string(root.join(".github/workflows/ci.yml")).unwrap();
    assert!(ci.contains("  pull_request:\n    branches: [master]\n"));
    assert!(ci.contains("workflow_dispatch:"));
    assert!(!ci.contains("  push:"));
    assert!(ci.contains("name: CI complete"));
}

#[test]
fn only_a_release_from_master_claims_latest() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let release = fs::read_to_string(root.join(".github/workflows/release.yml")).unwrap();

    // The repository holds only CLI releases, so a release from master is
    // the latest; a patch tagged on an older line must not displace it.
    assert!(release.contains("\n          make_latest: ${{ steps.line.outputs.latest }}\n"));
    assert_eq!(release.matches("make_latest:").count(), 1);
    assert!(release.contains("echo \"latest=true\" >> \"$GITHUB_OUTPUT\""));
    assert!(release.contains("echo \"latest=false\" >> \"$GITHUB_OUTPUT\""));
}

#[test]
fn user_facing_documentation_links_this_repository() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut documents = vec![
        root.join("README.md"),
        root.join("packaging/cli/README.md"),
        root.join("CHANGELOG.md"),
    ];
    documents.extend(
        fs::read_dir(root.join("docs/cli"))
            .unwrap()
            .map(|entry| entry.unwrap().path()),
    );

    for document in documents {
        let text = fs::read_to_string(&document).unwrap();
        // `maelic13/colosseum` is the desktop application's repository since
        // the swap (ADR-0012): it may be named, but the CLI's releases,
        // issues, sources and clone address are `maelic13/colosseum-cli`.
        for old in [
            "github.com/maelic13/colosseum/releases",
            "github.com/maelic13/colosseum/issues",
            "github.com/maelic13/colosseum/blob",
            "github.com/maelic13/colosseum/tree",
            "github.com/maelic13/colosseum.git",
            "github.com/maelic13/colosseum#readme",
            "github.com/maelic13/colosseum-gui",
            "tag%3Acli-v",
        ] {
            assert!(!text.contains(old), "{} links {old}", document.display());
        }
    }
}

/// A Windows console shows text in its own code page, not UTF-8, so a
/// non-ASCII symbol in anything the CLI prints arrives garbled: an em dash in
/// the standings read `ÔÇö`. What the CLI and the crates it prints through put
/// in code is ASCII; comments may use what they like.
#[test]
fn what_the_cli_prints_is_ascii() {
    let crates = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    // Inputs, not output: tests that non-ASCII input is read correctly.
    let inputs = ["# a comment, \u{e9} and all", "\"non-ascii-\u{17e}\""];
    let mut violations = Vec::new();
    for name in [
        "colosseum-core",
        "colosseum-application",
        "colosseum-uci",
        "colosseum-engine",
        "colosseum-cli",
    ] {
        for path in source_files(&crates.join(name).join("src")) {
            // The 1.x desktop application's scheduler, which only egui
            // renders, while it is still in this repository.
            if name == "colosseum-engine" && path.ends_with("scheduler.rs") {
                continue;
            }
            let text = fs::read_to_string(&path).unwrap();
            for (number, line) in text.lines().enumerate() {
                let code = code_part(line);
                if inputs.iter().any(|input| code.contains(input)) {
                    continue;
                }
                if !code.is_ascii() || has_non_ascii_escape(code) {
                    violations.push(format!(
                        "{}:{}: {}",
                        path.display(),
                        number + 1,
                        line.trim()
                    ));
                }
            }
        }
    }
    assert!(
        violations.is_empty(),
        "non-ASCII text the CLI may print:\n{}",
        violations.join("\n")
    );
}

/// A line without its `//` comment, if it has one outside a string literal.
fn code_part(line: &str) -> &str {
    let mut in_string = false;
    let mut previous = ' ';
    for (index, character) in line.char_indices() {
        if character == '"' && previous != '\\' {
            in_string = !in_string;
        } else if !in_string && line[index..].starts_with("//") {
            return &line[..index];
        }
        previous = character;
    }
    line
}

/// Whether `code` writes a non-ASCII character as a `\u{...}` escape.
fn has_non_ascii_escape(code: &str) -> bool {
    code.split("\\u{").skip(1).any(|rest| {
        rest.split_once('}')
            .and_then(|(hex, _)| u32::from_str_radix(hex, 16).ok())
            .is_some_and(|value| value > 0x7f)
    })
}

fn source_files(directory: &std::path::Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for entry in fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            files.extend(source_files(&path));
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }
    files
}
