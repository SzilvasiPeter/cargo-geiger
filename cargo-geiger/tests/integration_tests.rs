#![forbid(unsafe_code)]
#![forbid(warnings)]

mod context;
mod run;

use self::context::Context;
use self::run::run_geiger_with;

use insta::assert_snapshot;
use rstest::rstest;
use std::process::Output;

#[rstest(
    name,
    case("test1_package_with_no_deps"),
    case("test2_package_with_shallow_deps"),
    case("test3_package_with_nested_deps"),
    case("test4_workspace_with_top_level_package"),
    case("test5_workspace_with_virtual_manifest"),
    case("test6_cargo_lock_out_of_date"),
    case("test7_package_with_patched_dep"),
    case("test8_package_with_build_rs_no_deps"),
    case("test9_package_with_git_deps"),
    case("test10_package_with_docs_include_str")
)]
fn test_package(name: &str) {
    better_panic::install();

    let (result, context) = run_geiger(name);

    let stderr_filename = format!("{}.stderr", name);
    let stderr = String::from_utf8(result.stderr)
        .expect("output should have been valid utf-8");

    if !stderr.is_empty() {
        let manifest_path_regex = regex::Regex::new(r"`([^`]+).toml`").unwrap();
        let artifact_json_blob_regex = regex::Regex::new(
            r#"\{"\$message_type":"artifact","artifact":.*"emit":.*\}"#,
        )
        .unwrap();
        let temp_dir = context.path.to_string_lossy();

        let stderr = manifest_path_regex.replace(&stderr, "`{MANIFEST_PATH}`");
        let stderr = artifact_json_blob_regex
            .replace_all(&stderr, "`{ARTIFACT_JSON_BLOB}`");
        let stderr = stderr.replace(temp_dir.as_ref(), "{TEMP_DIR}");

        // On Windows, cargo outputs paths with a `\\?\` prefix and backslash separators.
        // Strip the prefix and normalize to forward slashes so that snapshots are consistent across platforms.
        let stderr = stderr.replace(r"\\?\", "").replace('\\', "/");

        assert_snapshot!(stderr_filename, stderr);
    }

    let stdout_filename = format!("{}.stdout", name);
    let stdout = String::from_utf8(result.stdout)
        .expect("output should have been valid utf-8");
    assert_snapshot!(stdout_filename, stdout);

    if stderr.is_empty() {
        assert!(result.status.success(), "`cargo-geiger` failed");
    }

    fn run_geiger(test_name: &str) -> (Output, Context) {
        run_geiger_with(test_name, &["--color", "never"])
    }
}
