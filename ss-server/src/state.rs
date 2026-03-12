use std::sync::Arc;

use handler::Handler;

#[derive(Clone)]
pub struct ServerState {
    handler: Arc<Handler>,
}

impl ServerState {
    pub fn new(handler: Arc<Handler>) -> Self {
        Self { handler }
    }

    pub fn handler(&self) -> &Arc<Handler> {
        &self.handler
    }
}
