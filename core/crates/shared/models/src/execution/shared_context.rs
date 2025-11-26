use super::context::RequestContext;
use std::sync::{Arc, Mutex};

pub type SharedRequestContext = Arc<Mutex<RequestContext>>;

impl From<RequestContext> for SharedRequestContext {
    fn from(context: RequestContext) -> Self {
        Self::new(Mutex::new(context))
    }
}

impl From<Arc<Mutex<Self>>> for RequestContext {
    fn from(shared: Arc<Mutex<Self>>) -> Self {
        let guard = shared
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        guard.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use systemprompt_identifiers::{AgentName, ContextId, SessionId, TraceId};

    #[test]
    fn test_shared_context_creation() {
        let ctx = RequestContext::new(
            SessionId::new("sess-123".to_string()),
            TraceId::new("trace-456".to_string()),
            ContextId::new("ctx-789".to_string()),
            AgentName::new("test_agent".to_string()),
        );

        let shared = SharedRequestContext::from(ctx);
        let locked = shared.lock().unwrap();
        assert_eq!(locked.request.session_id.as_str(), "sess-123");
    }

    #[test]
    fn test_shared_context_mutation() {
        let ctx = RequestContext::new(
            SessionId::new("sess-123".to_string()),
            TraceId::new("trace-456".to_string()),
            ContextId::new("ctx-789".to_string()),
            AgentName::new("test_agent".to_string()),
        );

        let shared = SharedRequestContext::from(ctx);

        {
            let mut locked = shared.lock().unwrap();
            locked.request.session_id = SessionId::new("sess-updated".to_string());
        }

        {
            let locked = shared.lock().unwrap();
            assert_eq!(locked.request.session_id.as_str(), "sess-updated");
        }
    }

    #[test]
    fn test_shared_context_clone() {
        let ctx = RequestContext::new(
            SessionId::new("sess-123".to_string()),
            TraceId::new("trace-456".to_string()),
            ContextId::new("ctx-789".to_string()),
            AgentName::new("test_agent".to_string()),
        );

        let shared1 = SharedRequestContext::from(ctx);
        let shared2 = shared1.clone();

        {
            let mut locked = shared1.lock().unwrap();
            locked.request.session_id = SessionId::new("sess-updated".to_string());
        }

        {
            let locked = shared2.lock().unwrap();
            assert_eq!(locked.request.session_id.as_str(), "sess-updated");
        }
    }
}
