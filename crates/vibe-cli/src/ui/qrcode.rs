use anyhow::Result;

use crate::auth::{PendingTerminalAuth, render_auth_qr};

pub fn auth_qr_text(pending: &PendingTerminalAuth) -> Result<String> {
    Ok(render_auth_qr(pending)?)
}
