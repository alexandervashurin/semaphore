//! db_lib модуль
//!
//! Замена Go db_lib пакета

pub mod access_key_installer;
pub mod ansible_app;
pub mod ansible_playbook;
pub mod app_factory;
pub mod cmd_git_client;
pub mod git_client_factory;
pub mod go_git_client;
pub mod local_app;
pub mod shell_app;
pub mod terraform_app;
pub mod types;

pub use access_key_installer::{
    AccessKeyInstallerImpl, AccessKeyInstallerTrait,
    DbAccessKey, DbAccessKeyOwner, DbAccessKeyRole, DbAccessKeyType,
    DbAccessKeySourceStorageType, DbLoginPassword, DbSshKey,
};

pub use ansible_app::{AnsibleApp, AnsiblePlaybook as AnsiblePlaybookStruct, GalaxyRequirementsType};
pub use ansible_playbook::AnsiblePlaybook;
pub use app_factory::create_app;
pub use cmd_git_client::{
    CmdGitClient, GitClient, GitRepository, GitRepositoryDirType,
    DbRepository,
};
pub use git_client_factory::{
    create_default_git_client, create_go_git_client, create_cmd_git_client,
    create_git_client, GitClientType,
};
pub use go_git_client::GoGitClient;
pub use local_app::{LocalApp, LocalAppRunningArgs, LocalAppInstallingArgs, AccessKeyInstaller};
pub use shell_app::ShellApp;
pub use terraform_app::TerraformApp;
pub use types::{TerraformInventoryAlias, TerraformInventoryState};
