//! Telegram Desktop's look, ported value for value: its fonts, its icon
//! masks, its palette keys and its style metrics (`*.style` files, the
//! `st::` namespace in tdesktop). The main window draws from here so it
//! matches tdesktop pixel for pixel (`docs/decisions/tdesktop-pixel-contract.md`).
#![deny(clippy::indexing_slicing)]

pub(crate) mod colors;
pub(crate) mod fonts;
pub(crate) mod icon;
pub(crate) mod st;

pub(crate) use colors::td;
pub(crate) use icon::{TdIcon, td_icon};
