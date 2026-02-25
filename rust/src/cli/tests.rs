//! Тесты для CLI

#[cfg(test)]
mod tests {
    use crate::cli::Cli;
    use clap::Parser;

    #[test]
    fn test_cli_server_command() {
        let cli = Cli::parse_from(["semaphore", "server", "--port", "8080"]);
        match cli.command {
            crate::cli::Commands::Server(args) => {
                assert_eq!(args.port, 8080);
            }
            _ => panic!("Ожидалась команда server"),
        }
    }

    #[test]
    fn test_cli_user_add_command() {
        let cli = Cli::parse_from([
            "semaphore",
            "user",
            "add",
            "--username",
            "test",
            "--name",
            "Test User",
            "--email",
            "test@example.com",
            "--password",
            "secret",
        ]);

        match cli.command {
            crate::cli::Commands::User(args) => {
                match args.command {
                    crate::cli::UserCommands::Add(add_args) => {
                        assert_eq!(add_args.username, "test");
                        assert_eq!(add_args.email, "test@example.com");
                    }
                    _ => panic!("Ожидалась команда add"),
                }
            }
            _ => panic!("Ожидалась команда user"),
        }
    }

    #[test]
    fn test_cli_version_command() {
        let cli = Cli::parse_from(["semaphore", "version"]);
        match cli.command {
            crate::cli::Commands::Version => {}
            _ => panic!("Ожидалась команда version"),
        }
    }
}
