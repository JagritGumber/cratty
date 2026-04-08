use cratty_core::AppConfig;

#[test]
fn default_config_has_font_family() {
    let cfg = AppConfig::default();
    assert_eq!(cfg.font_family, "JetBrains Mono");
}

#[test]
fn default_config_has_font_size() {
    let cfg = AppConfig::default();
    assert!((cfg.font_size - 14.0).abs() < 0.01);
}

#[test]
fn default_config_has_theme() {
    let cfg = AppConfig::default();
    assert_eq!(cfg.theme, "default-dark");
}

#[test]
fn default_config_has_scrollback() {
    let cfg = AppConfig::default();
    assert_eq!(cfg.scrollback_lines, 10_000);
}

#[test]
fn default_config_quake_mode_enabled() {
    let cfg = AppConfig::default();
    assert!(cfg.quake_mode.enabled);
    assert_eq!(cfg.quake_mode.hotkey, "F12");
    assert!((cfg.quake_mode.height_percent - 0.4).abs() < 0.01);
}

#[test]
fn default_config_has_shell_profiles() {
    let cfg = AppConfig::default();
    assert!(!cfg.shell_profiles.is_empty());
}

#[test]
fn config_round_trips_through_yaml() {
    let cfg = AppConfig::default();
    let yaml = serde_yaml::to_string(&cfg).expect("serialize");
    let restored: AppConfig =
        serde_yaml::from_str(&yaml).expect("deserialize");
    assert_eq!(restored.font_family, cfg.font_family);
    assert_eq!(restored.scrollback_lines, cfg.scrollback_lines);
}

#[test]
fn config_round_trips_through_json() {
    let cfg = AppConfig::default();
    let json = serde_json::to_string(&cfg).expect("serialize");
    let restored: AppConfig =
        serde_json::from_str(&json).expect("deserialize");
    assert_eq!(restored.theme, cfg.theme);
    assert!((restored.font_size - cfg.font_size).abs() < 0.01);
}

#[test]
fn config_deserializes_with_missing_fields() {
    let yaml = "font_size: 16.0\n";
    let cfg: AppConfig =
        serde_yaml::from_str(yaml).expect("partial deserialize");
    assert!((cfg.font_size - 16.0).abs() < 0.01);
    assert_eq!(cfg.font_family, "JetBrains Mono");
}

#[test]
fn config_load_missing_file_returns_default() {
    let path = std::path::Path::new("/nonexistent/path/config.yaml");
    let cfg = AppConfig::load(path).expect("fallback to default");
    assert_eq!(cfg.font_family, "JetBrains Mono");
}
