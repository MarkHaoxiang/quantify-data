use tokio::runtime::Runtime;

use crate::server::grpc::QuantifyDataServerImpl;

pub struct QuantifyApp {
    runtime: Runtime,  
    server: QuantifyDataServerImpl,
    gui: QuantifyGUI
}

impl QuantifyApp {

    pub fn new(runtime: Runtime, server: QuantifyDataServerImpl) -> QuantifyApp {
        QuantifyApp {
            runtime: runtime,
            server: server,
            gui: QuantifyGUI {}
        }
    }

    pub fn run(self, server_addr: std::net::SocketAddr) -> Result<(), eframe::Error> {
        // Setup gRPC
        let _enter = self.runtime.enter();
        std::thread::spawn(move || {
            self.runtime.block_on(self.server.start_service(server_addr))
        });

        // Launch GUI app on the main thread
        let options = eframe::NativeOptions {
            maximized: true,
            ..Default::default()
        };

        eframe::run_native(
            "Quantify",
            options,
            Box::new(|_cc| {
                Box::new(self.gui)
            }),
        )

        // Launch gRPC service
    }
}

pub struct QuantifyGUI {

}

impl eframe::App for QuantifyGUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Quantify");
            ui.horizontal(|ui| {
                let name_label = ui.label("Your name: ");
            });
            if ui.button("Click each year").clicked() {
            }
            ui.label(format!("Hello"));

        });
    }
}