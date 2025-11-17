use async_trait::async_trait;
use module_core::{Module, ModuleCtx};
use tracing::info;

pub struct Rest {
    ctx: ModuleCtx,
}

impl Rest {
    pub fn new(ctx: ModuleCtx) -> Self {
        Rest { ctx }
    }
}

#[async_trait]
impl Module for Rest {
    async fn run(&mut self) -> Result<(), ()> {
        info!("REST module started");
        // Placeholder for REST server logic
        Ok(())
    }
}
