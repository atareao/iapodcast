mod mastodon;
mod telegram;

pub use mastodon::{
    Mastodon,
    get_mastodon_client
};
pub use telegram::{
    Telegram,
    get_telegram_client
};
