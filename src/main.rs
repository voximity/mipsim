use app::App;

mod app;
mod assembler;
mod simulator;
mod util;

fn main() {
    eframe::run_native(
        "mipsim",
        eframe::NativeOptions::default(),
        Box::new(|_| {
            let app = Box::<App>::default();
            app.output.log.tx.send("Welcome to mipsim!".into()).unwrap();
            app
        }),
    )
    .unwrap();
}
