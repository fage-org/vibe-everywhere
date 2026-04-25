//! File Operations Module
//!
//! Handles file system operations with workspace boundary validation.

mod handlers;
#[cfg(test)]
mod tests;

#[cfg(test)]
pub(crate) use crate::error::DaemonError;
#[cfg(test)]
pub(crate) use ve_shared::models::{FileTreeNode, FileType};

pub use handlers::FileOps;
