use crate::server::grpc::QuantifyDataServerImpl;

pub struct QuantifyApp {
    pub server: QuantifyDataServerImpl
}

impl QuantifyApp {

    pub fn run(self) -> Result<(), eframe::Error> {
        let options = eframe::NativeOptions {
            initial_window_size: Some(egui::vec2(320.0, 240.0)),
            ..Default::default()
        };
        eframe::run_native(
            "Quantify",
            options,
            Box::new(|cc| {
                Box::new(self)
            }),
        )
    }
}

impl eframe::App for QuantifyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My egui Application");
            ui.horizontal(|ui| {
                let name_label = ui.label("Your name: ");
            });
            if ui.button("Click each year").clicked() {
            }
            ui.label(format!("Hello"));

        });
    }
}