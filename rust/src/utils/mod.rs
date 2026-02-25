//! Утилиты и вспомогательные функции

pub mod conv;
pub mod common_errors;

pub use conv::{convert_float_to_int_if_possible, struct_to_flat_map};
pub use common_errors::{
    get_error_context, new_user_error, InvalidSubscriptionError, UserVisibleError,
};
