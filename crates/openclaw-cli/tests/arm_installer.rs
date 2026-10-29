//! The ARM installer must hand `gateway install` an explicit config path.
//!
//! Real defect this guards: the script ran onboarding as the invoking user, so
//! config landed in `$HOME/.openclaw/openclaw.json`, then invoked
//! `sudo -E openclaw gateway install` without `--config`. `sudo` changes `HOME` to
//! `/root`, so the service step could not find the config and aborted with
//! "no configuration file found" after the binaries were already installed.

const INSTALLER: &str = include_str!("../../install-linux-arm64.sh");

#[test]
fn service_install_passes_an_explicit_config_path() {
    let service_block = INSTALLER
        .split("Installing the Gateway service")
        .nth(1)
        .expect("installer must have a service section");

    assert!(
        service_block.contains("--config"),
        "gateway install must receive --config, otherwise sudo's HOME change hides it:\n{service_block}"
    );
    assert!(
        service_block.contains("CONFIG_PATH"),
        "the explicit path must be the resolved CONFIG_PATH:\n{service_block}"
    );
}

#[test]
fn state_dir_is_exported_for_every_cli_call() {
    // Both onboarding and the service install must see the same state dir.
    assert!(
        INSTALLER.contains("OPENCLAW_STATE_DIR="),
        "installer must pin the state dir for child processes"
    );
}

#[test]
fn installer_refuses_unsupported_architectures() {
    assert!(INSTALLER.contains("aarch64|arm64"));
    assert!(INSTALLER.contains("unsupported architecture"));
}

#[test]
fn installer_never_writes_a_literal_secret() {
    for forbidden in ["TOKEN=your", "API_KEY=your", "bot_token_here"] {
        assert!(
            !INSTALLER.contains(forbidden),
            "installer must not embed placeholder secrets: {forbidden}"
        );
    }
}
