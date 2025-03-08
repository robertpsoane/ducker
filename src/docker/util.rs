use bollard::{Docker, API_DEFAULT_VERSION};
use color_eyre::eyre::{Context, Result};
use std::env;

use super::container::DockerContainer;

#[derive(Debug, PartialEq)]
enum DockerEndpoint {
    Tcp(String),
    Unix(String),
}

impl DockerEndpoint {
    fn from_env_or_default(default_socket: &str, override_host: Option<&str>) -> Self {
        // override_host
        if let Some(host) = override_host {
            return if host.starts_with("tcp://") {
                DockerEndpoint::Tcp(host.to_string())
            } else if host.starts_with("unix://") {
                DockerEndpoint::Unix(host[7..].to_string())
            } else {
                DockerEndpoint::Unix(default_socket.to_string())
            };
        }

        // Then try DOCKER_HOST environment variable
        if let Ok(host) = env::var("DOCKER_HOST") {
            if host.starts_with("tcp://") {
                DockerEndpoint::Tcp(host)
            } else if host.starts_with("unix://") {
                DockerEndpoint::Unix(host[7..].to_string())
            } else {
                DockerEndpoint::Unix(default_socket.to_string())
            }
        } else {
            // Fall back to default socket path
            DockerEndpoint::Unix(default_socket.to_string())
        }
    }

    async fn connect(&self) -> Result<Docker> {
        let docker = match self {
            DockerEndpoint::Tcp(host) => {
                bollard::Docker::connect_with_http(host, 120, API_DEFAULT_VERSION)
                    .with_context(|| format!("unable to connect to docker host {host}"))?
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

pub async fn new_local_docker_connection(socket_path: &str, docker_host: Option<&str>) -> Result<Docker> {
    DockerEndpoint::from_env_or_default(socket_path, docker_host).connect().await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::{remove_var, set_var};

    fn reset_docker_host() {
        remove_var("DOCKER_HOST");
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
        assert_eq!(endpoint, DockerEndpoint::Unix("/custom/docker.sock".to_string()));
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
        assert_eq!(endpoint, DockerEndpoint::Unix("/custom/docker.sock".to_string()));
    }

    #[test]
    fn test_docker_endpoint_cli_overrides_env() {
        reset_docker_host();
        let default_socket = "/var/run/docker.sock";
        let env_host = "tcp://1.2.3.4:2375";
        let cli_host = "unix:///custom/docker.sock";
        set_var("DOCKER_HOST", env_host);
        
        let endpoint = DockerEndpoint::from_env_or_default(default_socket, Some(cli_host));
        assert_eq!(endpoint, DockerEndpoint::Unix("/custom/docker.sock".to_string()));
    }

    #[test]
    fn test_docker_endpoint_fallback_to_default() {
        reset_docker_host();
        let default_socket = "/var/run/docker.sock";
        
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

    // Mock tests for connection
    #[tokio::test]
    async fn test_connection_error_handling() {
        // This test verifies error handling without requiring a Docker daemon
        let invalid_endpoint = DockerEndpoint::Tcp("tcp://invalid:1234".to_string());
        let result = invalid_endpoint.connect().await;
        assert!(result.is_err());
    }
}
