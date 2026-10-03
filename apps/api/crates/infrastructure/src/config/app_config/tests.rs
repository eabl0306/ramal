#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests fail loudly on purpose; a panic here is the assertion"
)]
#![allow(
    clippy::result_large_err,
    reason = "figment::Jail fixes the closure signature and its error type"
)]

use figment::Jail;

use super::{AppConfig, ConfigError};

/// `just` carga `.env` en el entorno de los tests: sin limpiar, una variable
/// de la maquina cambiaria el resultado.
fn load(jail: &mut Jail) -> Result<AppConfig, ConfigError> {
    jail.clear_env();
    AppConfig::from_env()
}

#[test]
fn falls_back_to_defaults_when_the_environment_is_empty() {
    Jail::expect_with(|jail| {
        let config = load(jail).expect("defaults should load");
        assert_eq!(config, AppConfig::default());
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.telemetry.log_level, "info");
        assert_eq!(config.database.max_connections, 5);
        assert!(!config.server.swagger_ui);
        Ok(())
    });
}

#[test]
fn reads_nested_keys_from_double_underscore_variables() {
    Jail::expect_with(|jail| {
        jail.clear_env();
        jail.set_env("GIT_RUDDER__SERVER__HOST", "0.0.0.0");
        jail.set_env("GIT_RUDDER__SERVER__PORT", "9090");
        jail.set_env("GIT_RUDDER__DATABASE__URL", "postgres://u:p@db:5432/x");
        jail.set_env("GIT_RUDDER__DATABASE__MAX_CONNECTIONS", "9");
        jail.set_env("GIT_RUDDER__DATABASE__ACQUIRE_TIMEOUT_MS", "1500");
        jail.set_env("GIT_RUDDER__TELEMETRY__LOG_LEVEL", "debug");
        jail.set_env("GIT_RUDDER__SERVER__SWAGGER_UI", "true");
        let config = AppConfig::from_env().expect("environment should load");
        assert!(config.server.swagger_ui);
        assert_eq!(config.database.url, "postgres://u:p@db:5432/x");
        assert_eq!(config.database.max_connections, 9);
        assert_eq!(config.database.acquire_timeout_ms, 1500);
        assert_eq!(config.telemetry.log_level, "debug");
        assert_eq!(
            config.socket_addr().expect("address should resolve"),
            "0.0.0.0:9090".parse().expect("literal is valid")
        );
        Ok(())
    });
}

#[test]
fn rejects_a_port_that_is_not_a_number() {
    Jail::expect_with(|jail| {
        jail.clear_env();
        jail.set_env("GIT_RUDDER__SERVER__PORT", "not-a-port");
        assert!(matches!(
            AppConfig::from_env(),
            Err(ConfigError::Invalid(_))
        ));
        Ok(())
    });
}

#[test]
fn rejects_a_key_it_does_not_know() {
    Jail::expect_with(|jail| {
        jail.clear_env();
        jail.set_env("GIT_RUDDER__DATABASE__PASSWORD", "secret");
        let error = AppConfig::from_env().expect_err("unknown keys must fail");
        assert!(error.to_string().contains("invalid configuration"));
        Ok(())
    });
}

#[test]
fn ignores_variables_without_the_double_underscore_prefix() {
    Jail::expect_with(|jail| {
        jail.clear_env();
        // Las de los tests de PostgreSQL llevan un solo guion bajo a proposito.
        jail.set_env("GIT_RUDDER_TEST_DATABASE_URL", "postgres://elsewhere");
        assert_eq!(
            AppConfig::from_env().expect("the variable is not ours"),
            AppConfig::default()
        );
        Ok(())
    });
}
