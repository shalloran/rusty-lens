// unified error type for parser and tui

use anyhow::Error;

pub type Result<T> = std::result::Result<T, Error>;
