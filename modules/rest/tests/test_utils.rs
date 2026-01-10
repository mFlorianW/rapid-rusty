// SPDX-FileCopyrightText: 2026 All contributors
//
// SPDX-License-Identifier: GPL-2.0-or-later

use module_core::{Module, ModuleCtx};
use rest::Rest;
use tokio::task::JoinHandle;

/// Creates and runs the REST module in a separate Tokio task.
/// # Arguments
/// * `ctx` - The module context to be used by the REST module.
/// # Returns
/// A JoinHandle that resolves to a Result indicating the success or failure of the module's execution
pub fn create_module(ctx: ModuleCtx) -> JoinHandle<Result<(), ()>> {
    tokio::spawn(async move {
        let mut rest = Rest::new(ctx);
        rest.run().await
    })
}
