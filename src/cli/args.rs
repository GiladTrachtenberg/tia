use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: ProviderCommand,
}

#[derive(Subcommand, Debug)]
pub enum ProviderCommand {
    Cloudflare {
        #[command(subcommand)]
        command: CloudflareCommand,
    },
}

#[derive(Subcommand, Debug)]
pub enum CloudflareCommand {
    Discover(DiscoverArgs),
    Generate(GenerateArgs),
    Diff(DiffArgs),
}

#[derive(clap::Args, Debug)]
pub struct DiscoverArgs {
    #[arg(long, env = "CLOUDFLARE_API_TOKEN", hide_env_values = true)]
    pub token: Option<String>,

    #[arg(long, env = "CLOUDFLARE_ZONE_ID")]
    pub zone: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    use serial_test::serial;

    #[test]
    fn test_discover_args_token_from_flag() {
        let cli = Cli::parse_from(["tia", "cloudflare", "discover", "--token=test_token"]);

        if let ProviderCommand::Cloudflare {
            command: CloudflareCommand::Discover(args),
        } = cli.command
        {
            assert_eq!(args.token, Some("test_token".to_string()));
        } else {
            panic!(
                "Expected Cloudflare Discover command, got {:?}",
                cli.command
            );
        }
    }

    #[test]
    fn test_discover_args_zone_from_flag() {
        let cli = Cli::parse_from(["tia", "cloudflare", "discover", "--zone=example.com"]);

        if let ProviderCommand::Cloudflare {
            command: CloudflareCommand::Discover(args),
        } = cli.command
        {
            assert_eq!(args.zone, Some("example.com".to_string()));
        } else {
            panic!(
                "Expected Cloudflare Discover command, got {:?}",
                cli.command
            );
        }
    }

    #[test]
    fn test_discover_args_both_token_and_zone() {
        let cli = Cli::parse_from([
            "tia",
            "cloudflare",
            "discover",
            "--token=my_token",
            "--zone=my_zone",
        ]);

        if let ProviderCommand::Cloudflare {
            command: CloudflareCommand::Discover(args),
        } = cli.command
        {
            assert_eq!(args.token, Some("my_token".to_string()));
            assert_eq!(args.zone, Some("my_zone".to_string()));
        } else {
            panic!(
                "Expected Cloudflare Discover command, got {:?}",
                cli.command
            );
        }
    }

    #[test]
    #[serial]
    fn test_discover_args_no_flags_provided() {
        let token_backup = std::env::var("CLOUDFLARE_API_TOKEN").ok();
        let zone_backup = std::env::var("CLOUDFLARE_ZONE_ID").ok();
        unsafe {
            std::env::remove_var("CLOUDFLARE_API_TOKEN");
            std::env::remove_var("CLOUDFLARE_ZONE_ID");
        }

        let cli = Cli::parse_from(["tia", "cloudflare", "discover"]);

        unsafe {
            if let Some(token) = token_backup {
                std::env::set_var("CLOUDFLARE_API_TOKEN", token);
            }
            if let Some(zone) = zone_backup {
                std::env::set_var("CLOUDFLARE_ZONE_ID", zone);
            }
        }

        if let ProviderCommand::Cloudflare {
            command: CloudflareCommand::Discover(args),
        } = cli.command
        {
            assert!(args.token.is_none());
            assert!(args.zone.is_none());
        } else {
            panic!(
                "Expected Cloudflare Discover command, got {:?}",
                cli.command
            );
        }
    }

    #[test]
    #[serial]
    fn test_cli_flag_takes_precedence_over_env() {
        let token_backup = std::env::var("CLOUDFLARE_API_TOKEN").ok();

        unsafe {
            std::env::set_var("CLOUDFLARE_API_TOKEN", "env_token");
        }

        let cli = Cli::parse_from(["tia", "cloudflare", "discover", "--token=cli_token"]);

        unsafe {
            match token_backup {
                Some(token) => std::env::set_var("CLOUDFLARE_API_TOKEN", token),
                None => std::env::remove_var("CLOUDFLARE_API_TOKEN"),
            }
        }

        if let ProviderCommand::Cloudflare {
            command: CloudflareCommand::Discover(args),
        } = cli.command
        {
            assert_eq!(args.token, Some("cli_token".to_string()));
        } else {
            panic!(
                "Expected Cloudflare Discover command, got {:?}",
                cli.command
            );
        }
    }

    #[test]
    #[serial]
    fn test_zone_from_env_var_fallback() {
        let zone_backup = std::env::var("CLOUDFLARE_ZONE_ID").ok();

        unsafe {
            std::env::set_var("CLOUDFLARE_ZONE_ID", "env_zone_id_123");
        }

        let cli = Cli::parse_from(["tia", "cloudflare", "discover"]);

        unsafe {
            match zone_backup {
                Some(zone) => std::env::set_var("CLOUDFLARE_ZONE_ID", zone),
                None => std::env::remove_var("CLOUDFLARE_ZONE_ID"),
            }
        }

        if let ProviderCommand::Cloudflare {
            command: CloudflareCommand::Discover(args),
        } = cli.command
        {
            assert_eq!(args.zone, Some("env_zone_id_123".to_string()));
        } else {
            panic!(
                "Expected Cloudflare Discover command, got {:?}",
                cli.command
            );
        }
    }

    #[test]
    #[serial]
    fn test_zone_cli_flag_takes_precedence_over_env() {
        let zone_backup = std::env::var("CLOUDFLARE_ZONE_ID").ok();

        unsafe {
            std::env::set_var("CLOUDFLARE_ZONE_ID", "env_zone_id");
        }

        let cli = Cli::parse_from(["tia", "cloudflare", "discover", "--zone=cli_zone_id"]);

        unsafe {
            match zone_backup {
                Some(zone) => std::env::set_var("CLOUDFLARE_ZONE_ID", zone),
                None => std::env::remove_var("CLOUDFLARE_ZONE_ID"),
            }
        }

        if let ProviderCommand::Cloudflare {
            command: CloudflareCommand::Discover(args),
        } = cli.command
        {
            assert_eq!(args.zone, Some("cli_zone_id".to_string()));
        } else {
            panic!(
                "Expected Cloudflare Discover command, got {:?}",
                cli.command
            );
        }
    }
}

#[derive(clap::Args, Debug)]
pub struct GenerateArgs {
    // TODO: implement
}

#[derive(clap::Args, Debug)]
pub struct DiffArgs {
    // TODO: implement
}
