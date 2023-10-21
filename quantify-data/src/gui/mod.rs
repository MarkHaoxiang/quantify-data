use std::sync::Arc;

use quantifylib::executor::Executor;
use tokio::runtime::{Runtime, self};

use crate::server::grpc::QuantifyDataServerImpl;

mod menu_bar;
/// Bundled quantify application with GUI
/// Since this contains a direct reference to the executor
/// We can avoid using gRPC
pub struct QuantifyApp {
    // Shared runtime for use in calling async functions - probably using executor.
    runtime: Runtime,  
    // Executor (shared with gRPC server)
    executor: Arc<Executor>,
}

impl QuantifyApp {
    /// Starts the application
    /// 
    /// # Arguments
    /// 
    /// * 'executor' - Executor to run tasks
    /// * 'grpc_server' - If provided, attempts to start a gRPC server at grpc_server_addr
    /// * 'grpc_server_addr' - See above
    pub fn run(executor: Arc<Executor>,
               grpc_server: Option<QuantifyDataServerImpl>,
               grpc_server_addr: Option<std::net::SocketAddr>) -> Result<(), eframe::Error> {
        let runtime: Runtime = runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        // gRPC server if needed
        if let Some(grpc_server) = grpc_server {
            if let Some(grpc_server_addr) = grpc_server_addr {
                runtime.spawn(grpc_server.start_service(grpc_server_addr));
            }
            else {
                panic!("Attempting to initialise grpc server without address");
            }
        }
        // Create quantify app
        let app = QuantifyApp {runtime:runtime, executor: executor};
        // Launch GUI app on the main thread
        let mut options = eframe::NativeOptions::default();
        options.maximized = true;

        eframe::run_native(
            "Quantify",
            options,
            Box::new(|_cc| {Box::new(app)}),
        )
    }
}

impl eframe::App for QuantifyApp {
    /// Definition of GUI
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, menu_bar::add_contents);

        // Main window section
        egui::CentralPanel::default().show(ctx, |ui| {
        });
    }
}
