//! Утилиты и вспомогательные функции

pub mod conv;
pub mod common_errors;
pub mod mailer;
pub mod encryption;
pub mod shell;

pub use conv::{convert_float_to_int_if_possible, struct_to_flat_map};
pub use common_errors::{
    get_error_context, new_user_error, InvalidSubscriptionError, UserVisibleError,
};
pub use mailer::{send_email, Email, MailerError, SmtpConfig, is_valid_email};
pub use encryption::{generate_private_key, EncryptionError, KeyPair};
pub use shell::{shell_quote, shell_strip_unsafe};
