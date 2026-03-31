//! Утилиты и вспомогательные функции

pub mod ansi;
pub mod app;
pub mod common_errors;
pub mod conv;
pub mod debug;
pub mod encryption;
pub mod error_logging;
pub mod mailer;
pub mod oidc_provider;
pub mod shell;
pub mod test_helpers;
pub mod version;

pub use ansi::{clear_from_ansi_codes, contains_ansi_codes, strip_ansi_and_trim};
pub use app::App;
pub use common_errors::{
    get_error_context, new_user_error, InvalidSubscriptionError, UserVisibleError,
};
pub use conv::{convert_float_to_int_if_possible, struct_to_flat_map};
pub use debug::{debug_thread_id, log_thread_id, thread_id};
pub use encryption::{generate_private_key, EncryptionError, KeyPair};
pub use error_logging::{
    log_debug_f, log_error, log_error_f, log_panic, log_panic_f, log_warning, log_warning_f,
};
pub use mailer::{is_valid_email, send_email, Email, MailerError, SmtpConfig};
pub use oidc_provider::{OidcEndpoint, OidcProvider};
pub use shell::{shell_quote, shell_strip_unsafe};
pub use test_helpers::{rand_bool, rand_email, rand_range, rand_string, rand_username};
pub use version::{get_commit, get_date, get_version, version, COMMIT, DATE, VER};
