//! The value on a net. Four-valued: driven 0, driven 1, floating, unknown.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize, Hash)]
pub enum Sig {
    #[serde(rename = "0")]
    L,
    #[serde(rename = "1")]
    H,
    /// Floating: nothing drives this line.
    #[serde(rename = "Z")]
    Z,
    /// Unknown or conflicting.
    #[serde(rename = "X")]
    X,
}

impl Sig {
    pub fn from_bool(b: bool) -> Sig {
        if b {
            Sig::H
        } else {
            Sig::L
        }
    }
    pub fn as_bool(self) -> Option<bool> {
        match self {
            Sig::H => Some(true),
            Sig::L => Some(false),
            _ => None,
        }
    }
}
