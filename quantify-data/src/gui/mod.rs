use tokio::runtime::Runtime;

use crate::server::grpc::QuantifyDataServerImpl;

mod menu_bar;
/// Bundled quantify application with GUI
pub struct QuantifyApp {
    runtime: Runtime,  
    server: QuantifyDataServerImpl,
    gui: QuantifyGUI
}

impl QuantifyApp {
    /// Creates a bundled Quantify App
    /// 
    /// # Arguments
    /// 
    /// * 'runtime' - Tokio runtime for async functions
    /// * 'server' - gRPC server implementation
    pub fn new(runtime: Runtime, server: QuantifyDataServerImpl) -> QuantifyApp {
        QuantifyApp {
            runtime: runtime,
            server: server,
            gui: QuantifyGUI {}
        }
    }

    /// Starts the runtime by launching an instance of the gRPC server and the UI
    /// 
    /// # Arguments
    /// 
    /// * 'server_addr' - gRPC server recieving socket
    pub fn run(self, server_addr: std::net::SocketAddr) -> Result<(), eframe::Error> {
        // Setup gRPC
        let _enter = self.runtime.enter();
        std::thread::spawn(move || {
            self.runtime.block_on(self.server.start_service(server_addr))
        });

        // Launch GUI app on the main thread
        let mut options = eframe::NativeOptions::default();
        options.maximized = true;

        eframe::run_native(
            "Quantify",
            options,
            Box::new(|_cc| {Box::new(self.gui)}),
        )
    }
}

/// Representation of the GUI display state
pub struct QuantifyGUI {

}

impl eframe::App for QuantifyGUI {
    /// Definition of GUI
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, menu_bar::add_contents);

        // Main window section
        egui::CentralPanel::default().show(ctx, |ui| {
        });
    }
}