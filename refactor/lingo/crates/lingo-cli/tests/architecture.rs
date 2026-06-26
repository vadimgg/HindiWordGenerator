#![allow(clippy::expect_used, clippy::panic)]

use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::process::Command;

const WORKSPACE_CRATES: [&str; 7] = [
    "lingo-domain",
    "lingo-application",
    "lingo-workspace-fs",
    "lingo-prompt",
    "lingo-audio",
    "lingo-artifacts",
    "lingo-cli",
];

#[test]
fn inward_dependencies_are_enforced() {
    let graph = WorkspaceGraph::load();
    graph.assert_exact("lingo-domain", &[]);
    graph.assert_exact("lingo-application", &["lingo-domain"]);
    graph.assert_exact("lingo-workspace-fs", &["lingo-application", "lingo-domain"]);
    graph.assert_exact("lingo-prompt", &["lingo-application", "lingo-domain"]);
    graph.assert_exact("lingo-audio", &["lingo-application", "lingo-domain"]);
    graph.assert_exact("lingo-artifacts", &["lingo-application", "lingo-domain"]);
    graph.assert_exact(
        "lingo-cli",
        &[
            "lingo-application",
            "lingo-artifacts",
            "lingo-audio",
            "lingo-domain",
            "lingo-prompt",
            "lingo-workspace-fs",
        ],
    );
}

struct WorkspaceGraph {
    dependencies: BTreeMap<String, BTreeSet<String>>,
}

impl WorkspaceGraph {
    fn load() -> Self {
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Cargo.toml");
        let output = Command::new(env!("CARGO"))
            .args([
                "metadata",
                "--format-version",
                "1",
                "--no-deps",
                "--manifest-path",
            ])
            .arg(&manifest)
            .output()
            .expect("cargo metadata should run");
        assert!(
            output.status.success(),
            "cargo metadata failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let metadata: Value =
            serde_json::from_slice(&output.stdout).expect("cargo metadata should be JSON");
        let workspace = WORKSPACE_CRATES
            .into_iter()
            .map(str::to_string)
            .collect::<BTreeSet<_>>();
        let mut dependencies = BTreeMap::new();
        for package in metadata["packages"]
            .as_array()
            .expect("metadata should contain packages")
        {
            let name = package["name"]
                .as_str()
                .expect("package should have a name")
                .to_string();
            if !workspace.contains(&name) {
                continue;
            }
            let edges = package["dependencies"]
                .as_array()
                .expect("package should contain dependencies")
                .iter()
                .filter_map(|dependency| dependency["name"].as_str())
                .filter(|dependency| workspace.contains(*dependency))
                .map(str::to_string)
                .collect::<BTreeSet<_>>();
            dependencies.insert(name, edges);
        }
        assert_eq!(
            dependencies.keys().cloned().collect::<BTreeSet<_>>(),
            workspace,
            "metadata did not expose the expected clean-slate workspace crates"
        );
        Self { dependencies }
    }

    fn assert_exact(&self, package: &str, expected: &[&str]) {
        let actual = self
            .dependencies
            .get(package)
            .expect("workspace package should be present");
        let expected = expected
            .iter()
            .map(|dependency| (*dependency).to_string())
            .collect::<BTreeSet<_>>();
        assert_eq!(
            actual, &expected,
            "workspace dependency drift for {package}; actual edges: {actual:?}, expected: {expected:?}"
        );
    }
}
