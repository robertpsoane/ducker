use bollard::{Docker, API_DEFAULT_VERSION};
use color_eyre::eyre::{Context, Result};
use std::{env, path::Path};

use super::container::DockerContainer;

#[derive(Debug, PartialEq)]
enum DockerEndpoint {
    Tcp(String),
    Unix(String),
    Tls(String), // URL with TCP scheme but secured with TLS
}

impl DockerEndpoint {
    fn from_env_or_default(default_socket: &str, override_host: Option<&str>) -> Self {
        // override_host
        if let Some(host) = override_host {
            return Self::parse_endpoint(host, default_socket);
        }

        // Then try DOCKER_HOST environment variable
        if let Ok(host) = env::var("DOCKER_HOST") {
            Self::parse_endpoint(&host, default_socket)
        } else {
            // Fall back to default socket path
            DockerEndpoint::Unix(default_socket.to_string())
        }
    }

    fn parse_endpoint(host: &str, default_socket: &str) -> Self {
        if host.starts_with("tcp://") {
            // Check if it's a TLS port (typically 2376)
            if host.contains(":2376") || env::var("DOCKER_TLS_VERIFY").is_ok() {
                DockerEndpoint::Tls(host.to_string())
            } else {
                DockerEndpoint::Tcp(host.to_string())
            }
        } else if host.starts_with("https://") {
            DockerEndpoint::Tls(host.to_string())
        } else if host.starts_with("unix://") {
            DockerEndpoint::Unix(host[7..].to_string())
        } else {
            DockerEndpoint::Unix(default_socket.to_string())
        }
    }

    async fn connect(&self) -> Result<Docker> {
        let docker = match self {
            DockerEndpoint::Tcp(host) => {
                bollard::Docker::connect_with_http(host, 120, API_DEFAULT_VERSION)
                    .with_context(|| format!("unable to connect to docker host {host}"))?
            }
            DockerEndpoint::Tls(host) => {
                // Get cert path from environment
                let cert_path = env::var("DOCKER_CERT_PATH").unwrap_or_else(|_| {
                    // Default to ~/.docker
                    let home = env::var("HOME").unwrap_or_else(|_| {
                        if cfg!(windows) {
                            env::var("USERPROFILE").unwrap_or_else(|_| ".".to_string())
                        } else {
                            ".".to_string()
                        }
                    });
                    format!("{}/.docker", home)
                });

                // Construct paths to required certificate files
                let cert_file = Path::new(&cert_path).join("cert.pem");
                let key_file = Path::new(&cert_path).join("key.pem");
                let ca_file = Path::new(&cert_path).join("ca.pem");

                // Check if TLS verification is enabled (default true)
                let tls_verify = env::var("DOCKER_TLS_VERIFY")
                    .map(|val| !val.is_empty() && val != "0")
                    .unwrap_or(true);

                // For TLS connections, we need to check if host starts with tcp:// instead of https://
                let host_url = if host.starts_with("tcp://") {
                    // Replace tcp:// with https:// for the Docker API client
                    format!("https://{}", &host[6..])
                } else {
                    host.to_string()
                };

                // Connect with SSL - provide CA file only when verification is enabled
                bollard::Docker::connect_with_ssl(
                    &host_url,
                    &key_file,
                    &cert_file,
                    if tls_verify { &ca_file } else { &cert_file }, // Use cert as CA when not verifying
                    120,
                    API_DEFAULT_VERSION,
                )
                .with_context(|| format!("unable to connect to TLS docker host {host}"))?
            }
            DockerEndpoint::Unix(socket) => {
                bollard::Docker::connect_with_socket(socket, 120, API_DEFAULT_VERSION)
                    .with_context(|| format!("unable to connect to docker socket {socket}"))?
            }
        };

        // Verify connection works by listing containers
        DockerContainer::list(&docker)
            .await
            .context("unable to connect to docker")?;

        Ok(docker)
    }
}

pub async fn new_local_docker_connection(
    socket_path: &str,
    docker_host: Option<&str>,
) -> Result<Docker> {
    DockerEndpoint::from_env_or_default(socket_path, docker_host)
        .connect()
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::{remove_var, set_var};

    // Helper function to reset DOCKER_HOST environment variable
    fn reset_docker_host() {
        remove_var("DOCKER_HOST");
        // Verify the environment variable is actually removed
        assert!(
            env::var("DOCKER_HOST").is_err(),
            "DOCKER_HOST environment variable should be unset"
        );
    }

    #[test]
    fn test_docker_endpoint_from_cli_tcp() {
        reset_docker_host();
        let default_socket = "/var/run/docker.sock";
        let cli_host = "tcp://1.2.3.4:2375";

        let endpoint = DockerEndpoint::from_env_or_default(default_socket, Some(cli_host));
        assert_eq!(endpoint, DockerEndpoint::Tcp(cli_host.to_string()));
    }

    #[test]
    fn test_docker_endpoint_from_cli_unix() {
        reset_docker_host();
        let default_socket = "/var/run/docker.sock";
        let cli_host = "unix:///custom/docker.sock";

        let endpoint = DockerEndpoint::from_env_or_default(default_socket, Some(cli_host));
        assert_eq!(
            endpoint,
            DockerEndpoint::Unix("/custom/docker.sock".to_string())
        );
    }

    #[test]
    fn test_docker_endpoint_from_env_tcp() {
        reset_docker_host();
        let default_socket = "/var/run/docker.sock";
        let env_host = "tcp://1.2.3.4:2375";
        set_var("DOCKER_HOST", env_host);

        let endpoint = DockerEndpoint::from_env_or_default(default_socket, None);
        assert_eq!(endpoint, DockerEndpoint::Tcp(env_host.to_string()));
    }

    #[test]
    fn test_docker_endpoint_from_env_unix() {
        reset_docker_host();
        let default_socket = "/var/run/docker.sock";
        let env_host = "unix:///custom/docker.sock";
        set_var("DOCKER_HOST", env_host);

        let endpoint = DockerEndpoint::from_env_or_default(default_socket, None);
        assert_eq!(
            endpoint,
            DockerEndpoint::Unix("/custom/docker.sock".to_string())
        );
    }

    #[test]
    fn test_docker_endpoint_cli_overrides_env() {
        reset_docker_host();
        let default_socket = "/var/run/docker.sock";
        let env_host = "tcp://1.2.3.4:2375";
        let cli_host = "unix:///custom/docker.sock";
        set_var("DOCKER_HOST", env_host);

        let endpoint = DockerEndpoint::from_env_or_default(default_socket, Some(cli_host));
        assert_eq!(
            endpoint,
            DockerEndpoint::Unix("/custom/docker.sock".to_string())
        );
    }

    #[test]
    fn test_docker_endpoint_fallback_to_default() {
        // Double-check that DOCKER_HOST is unset
        reset_docker_host();

        let default_socket = "/var/run/docker.sock";

        // Verify DOCKER_HOST is still unset before the test
        assert!(
            env::var("DOCKER_HOST").is_err(),
            "DOCKER_HOST should be unset before test"
        );

        let endpoint = DockerEndpoint::from_env_or_default(default_socket, None);
        assert_eq!(endpoint, DockerEndpoint::Unix(default_socket.to_string()));
    }

    #[test]
    fn test_docker_endpoint_invalid_format_fallback() {
        reset_docker_host();
        let default_socket = "/var/run/docker.sock";
        let invalid_host = "invalid://1.2.3.4:2375";
        set_var("DOCKER_HOST", invalid_host);

        let endpoint = DockerEndpoint::from_env_or_default(default_socket, None);
        assert_eq!(endpoint, DockerEndpoint::Unix(default_socket.to_string()));
    }

    #[test]
    fn test_docker_endpoint_tcp_with_tls_port() {
        reset_docker_host();
        let default_socket = "/var/run/docker.sock";
        let cli_host = "tcp://rapi.cwel.sh:2376";

        let endpoint = DockerEndpoint::from_env_or_default(default_socket, Some(cli_host));
        assert_eq!(endpoint, DockerEndpoint::Tls(cli_host.to_string()));
    }

    #[test]
    fn test_docker_endpoint_tls_verify_forces_tls() {
        reset_docker_host();
        let default_socket = "/var/run/docker.sock";
        let cli_host = "tcp://rapi.cwel.sh:2375"; // Normal non-TLS port
        set_var("DOCKER_TLS_VERIFY", "1");

        let endpoint = DockerEndpoint::from_env_or_default(default_socket, Some(cli_host));
        assert_eq!(endpoint, DockerEndpoint::Tls(cli_host.to_string()));

        // Clean up
        remove_var("DOCKER_TLS_VERIFY");
    }

    // Mock tests for connection
    #[tokio::test]
    async fn test_connection_error_handling() {
        // This test verifies error handling without requiring a Docker daemon
        let invalid_endpoint = DockerEndpoint::Tcp("tcp://invalid:1234".to_string());
        let result = invalid_endpoint.connect().await;
        assert!(result.is_err());
    }
}
