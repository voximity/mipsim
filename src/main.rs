use app::App;
use simulator::Processor;

mod app;
mod assembler;
mod simulator;
mod util;

fn main() {
    let (proc_tx, app_rx) = Processor::spawn();

    eframe::run_native(
        "mipsim",
        eframe::NativeOptions::default(),
        Box::new(|_| {
            let app = Box::new(App::new(proc_tx, app_rx));
            app.output.log.tx.send("Welcome to mipsim!".into()).unwrap();
            app
        }),
    )
    .unwrap();
}
