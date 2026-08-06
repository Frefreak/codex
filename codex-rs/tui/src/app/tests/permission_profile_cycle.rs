use super::*;
use crate::legacy_core::config::PermissionProfileCatalogEntry;
use codex_protocol::models::ActivePermissionProfile;
use pretty_assertions::assert_eq;
use tempfile::tempdir;

#[tokio::test]
async fn cycles_allowed_named_permission_profiles_and_wraps() -> Result<()> {
    let (mut app, _app_event_rx, _op_rx) = make_test_app_with_channels().await;
    let codex_home = tempdir()?;
    let selected_config = codex_home.path().join("work.config.toml");
    std::fs::write(
        &selected_config,
        r#"
default_permissions = "manual"

[permissions.manual.filesystem]
":root" = "read"

[permissions.blocked.filesystem]
":root" = "read"

[permissions.auto.filesystem]
":root" = "read"
":workspace_roots" = "write"
"#,
    )?;
    app.config.codex_home = codex_home.path().to_path_buf().abs();
    app.loader_overrides.user_config_path = Some(selected_config.abs());
    app.config.custom_permission_profiles = vec![
        PermissionProfileCatalogEntry {
            id: "manual".to_string(),
            description: None,
            allowed: true,
        },
        PermissionProfileCatalogEntry {
            id: "blocked".to_string(),
            description: None,
            allowed: false,
        },
        PermissionProfileCatalogEntry {
            id: "auto".to_string(),
            description: None,
            allowed: true,
        },
    ];
    assert!(
        app.apply_permission_profile_selection(PermissionProfileSelection {
            profile_id: "manual".to_string(),
            approval_policy: None,
            approvals_reviewer: None,
            display_label: "manual".to_string(),
        })
        .await
    );

    assert!(app.cycle_permission_profile().await);
    assert_eq!(
        (
            app.config.permissions.active_permission_profile(),
            app.chat_widget
                .config_ref()
                .permissions
                .active_permission_profile()
        ),
        (
            Some(ActivePermissionProfile::new("auto")),
            Some(ActivePermissionProfile::new("auto")),
        )
    );

    assert!(app.cycle_permission_profile().await);
    assert_eq!(
        app.config.permissions.active_permission_profile(),
        Some(ActivePermissionProfile::new("manual"))
    );

    app.config.custom_permission_profiles.truncate(1);
    assert!(!app.cycle_permission_profile().await);
    assert_eq!(
        app.config.permissions.active_permission_profile(),
        Some(ActivePermissionProfile::new("manual"))
    );

    Ok(())
}
