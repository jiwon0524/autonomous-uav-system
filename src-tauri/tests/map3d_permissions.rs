use serde_json::Value;
use std::collections::BTreeSet;

// Configuration checks only: a running WebView must still be tested separately.
#[test]
fn app_commands_are_listed_and_only_main_receives_them() {
    let main_source = include_str!("../src/main.rs");
    let manifest = include_str!("../build.rs");
    let mut commands = BTreeSet::new();
    for line in main_source.lines() {
        let trimmed = line.trim().trim_end_matches(',');
        if let Some((_, command)) = trimmed.split_once("::") {
            if !command.is_empty()
                && command
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_')
            {
                commands.insert(command.to_owned());
            }
        }
    }
    assert!(!commands.is_empty());
    for command in &commands {
        assert!(
            manifest.contains(&format!("\"{command}\"")),
            "missing app manifest command {command}"
        );
    }
    let capability: Value =
        serde_json::from_str(include_str!("../capabilities/main.json")).unwrap();
    assert_eq!(capability["webviews"], serde_json::json!(["main"]));
    assert!(capability.get("remote").is_none());
    let permissions = capability["permissions"].as_array().unwrap();
    for command in &commands {
        let expected = format!("allow-{}", command.replace('_', "-"));
        assert!(
            permissions.iter().any(|x| x.as_str() == Some(&expected)),
            "main missing {expected}"
        );
    }
    assert!(!permissions
        .iter()
        .any(|p| p.as_str().is_some_and(|s| s.contains('*'))));
    let config: Value = serde_json::from_str(include_str!("../tauri.conf.json")).unwrap();
    assert_eq!(
        config["app"]["security"]["capabilities"],
        serde_json::json!(["main"])
    );
}
