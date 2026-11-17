#![feature(default_field_values)]
#![feature(error_generic_member_access)]

mod default_field_values;
mod default_on_rewritten_types;
#[cfg(feature = "cli")]
mod derive_settings;
#[cfg(feature = "cli")]
mod diff_from_defaults;
mod from_vec_str;
mod graphemics;
mod init_compact;
#[cfg(feature = "cli")]
mod live_settings;
mod live_settings_secret_rotation;
mod make_df;
mod my_config_primitives;
mod scream_it;
mod settings_config_name;
mod settings_env_profile;
#[cfg(feature = "cli")]
mod settings_flatten_cross_module;
mod settings_missing_config_warns;
#[cfg(feature = "cli")]
mod settings_module;
#[cfg(feature = "cli")]
mod settings_no_default;
mod settings_required_placeholders;
#[cfg(feature = "cli")]
mod settings_schema_optional;
#[cfg(feature = "cli")]
mod settings_schema;
#[cfg(feature = "cli")]
mod test_unknown_field_warning;
mod wrap_err;
mod wrap_new;
