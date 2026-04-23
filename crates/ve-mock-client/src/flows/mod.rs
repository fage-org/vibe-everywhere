//! Flow definitions and registry

use std::sync::Arc;

use serde::Serialize;

use crate::test_context::TestContext;

/// Result of running a single flow
#[derive(Debug, Clone, Serialize)]
pub struct FlowResult {
    pub id: String,
    pub status: String, // "PASS", "FAIL", "SKIP"
    pub message: String,
    pub duration_secs: f64,
}

impl FlowResult {
    pub fn pass(id: &str, duration: f64) -> Self {
        Self {
            id: id.to_string(),
            status: "PASS".to_string(),
            message: "ok".to_string(),
            duration_secs: duration,
        }
    }

    pub fn fail(id: &str, message: &str) -> Self {
        Self {
            id: id.to_string(),
            status: "FAIL".to_string(),
            message: message.to_string(),
            duration_secs: 0.0,
        }
    }

    pub fn skipped(id: &str, reason: &str) -> Self {
        Self {
            id: id.to_string(),
            status: "SKIP".to_string(),
            message: reason.to_string(),
            duration_secs: 0.0,
        }
    }
}

/// A registered flow
pub struct Flow {
    pub id: String,
    pub description: String,
    pub requires_agent: bool,
    #[allow(clippy::type_complexity)]
    pub run_fn: fn(
        Arc<TestContext>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = FlowResult> + Send>>,
}

/// Registry of all test flows
pub struct FlowRegistry {
    flows: Vec<Flow>,
}

impl Default for FlowRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl FlowRegistry {
    pub fn new() -> Self {
        let mut registry = Self { flows: Vec::new() };
        registry.register_all();
        registry
    }

    pub fn list(&self) -> &[Flow] {
        &self.flows
    }

    fn register_all(&mut self) {
        self.register(Flow {
            id: "f1".to_string(),
            description: "Device registration & pairing".to_string(),
            requires_agent: false,
            run_fn: |ctx| Box::pin(crate::flows::f1_device_registration_pairing::run(ctx)),
        });

        self.register(Flow {
            id: "f2".to_string(),
            description: "Host & Workspace CRUD".to_string(),
            requires_agent: false,
            run_fn: |ctx| Box::pin(crate::flows::f2_host_workspace_crud::run(ctx)),
        });

        self.register(Flow {
            id: "f3".to_string(),
            description: "Session create & execute".to_string(),
            requires_agent: true,
            run_fn: |ctx| Box::pin(crate::flows::f3_session_create_execute::run(ctx)),
        });

        self.register(Flow {
            id: "f4".to_string(),
            description: "Session message flow".to_string(),
            requires_agent: false,
            run_fn: |ctx| Box::pin(crate::flows::f4_session_message::run(ctx)),
        });

        self.register(Flow {
            id: "f5".to_string(),
            description: "Session control (pause/restart)".to_string(),
            requires_agent: true,
            run_fn: |ctx| Box::pin(crate::flows::f5_session_control::run(ctx)),
        });

        self.register(Flow {
            id: "f6".to_string(),
            description: "Permission request/response".to_string(),
            requires_agent: true,
            run_fn: |ctx| Box::pin(crate::flows::f6_permission_request_response::run(ctx)),
        });

        self.register(Flow {
            id: "f7".to_string(),
            description: "Session archival".to_string(),
            requires_agent: true,
            run_fn: |ctx| Box::pin(crate::flows::f7_session_archival::run(ctx)),
        });

        self.register(Flow {
            id: "f8".to_string(),
            description: "File browsing".to_string(),
            requires_agent: true,
            run_fn: |ctx| Box::pin(crate::flows::f8_file_browsing::run(ctx)),
        });

        self.register(Flow {
            id: "f9".to_string(),
            description: "Archive browse & delete".to_string(),
            requires_agent: false,
            run_fn: |ctx| Box::pin(crate::flows::f9_archive_browse_delete::run(ctx)),
        });

        self.register(Flow {
            id: "f10".to_string(),
            description: "Settings — get/update notification preferences".to_string(),
            requires_agent: false,
            run_fn: |ctx| Box::pin(crate::flows::f10_settings::run(ctx)),
        });

        self.register(Flow {
            id: "f11".to_string(),
            description: "Daemon reconnection".to_string(),
            requires_agent: false,
            run_fn: |ctx| Box::pin(crate::flows::f11_daemon_reconnection::run(ctx)),
        });

        self.register(Flow {
            id: "f12".to_string(),
            description: "Background tasks".to_string(),
            requires_agent: false,
            run_fn: |ctx| Box::pin(crate::flows::f12_background_tasks::run(ctx)),
        });
    }

    fn register(&mut self, flow: Flow) {
        self.flows.push(flow);
    }
}

// ---- Flow implementations (stubs — to be implemented in individual files) ----

pub mod f10_settings;
pub mod f11_daemon_reconnection;
pub mod f12_background_tasks;
pub mod f1_device_registration_pairing;
pub mod f2_host_workspace_crud;
pub mod f3_session_create_execute;
pub mod f4_session_message;
pub mod f5_session_control;
pub mod f6_permission_request_response;
pub mod f7_session_archival;
pub mod f8_file_browsing;
pub mod f9_archive_browse_delete;
