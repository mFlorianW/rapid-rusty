use module_core::{EventBus, Module, ModuleCtx, test_helper::stop_module};
use rest::Rest;
use rocket::local::blocking::Client;
use tokio::task::JoinHandle;

fn create_module(ctx: ModuleCtx) -> JoinHandle<Result<(), ()>> {
    tokio::spawn(async move {
        let mut rest = Rest::new(ctx);
        rest.run().await
    })
}

#[tokio::test]
async fn get_session_request_ids() {
    let eb = EventBus::default();
    let mut rest = create_module(eb.context());

    stop_module(&eb, &mut rest).await;
}
