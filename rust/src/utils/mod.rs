//! Утилиты и вспомогательные функции

pub mod ansi;
pub mod app;
pub mod conv;
pub mod common_errors;
pub mod debug;
pub mod error_logging;
pub mod mailer;
pub mod encryption;
pub mod shell;
pub mod version;

pub use ansi::{clear_from_ansi_codes, contains_ansi_codes, strip_ansi_and_trim};
pub use app::App;
pub use conv::{convert_float_to_int_if_possible, struct_to_flat_map};
pub use common_errors::{
    get_error_context, new_user_error, InvalidSubscriptionError, UserVisibleError,
};
pub use debug::{debug_thread_id, log_thread_id, thread_id};
pub use error_logging::{
    log_debug_f, log_error, log_error_f, log_panic, log_panic_f, log_warning, log_warning_f,
};
pub use mailer::{send_email, Email, MailerError, SmtpConfig, is_valid_email};
pub use encryption::{generate_private_key, EncryptionError, KeyPair};
pub use shell::{shell_quote, shell_strip_unsafe};
pub use version::{version, get_version, get_commit, get_date, VER, COMMIT, DATE};
