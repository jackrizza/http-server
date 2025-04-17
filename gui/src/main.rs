#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#[cfg(not(target_os = "linux"))]
use std::{cell::RefCell, rc::Rc};
use std::{
    io::{BufRead, BufReader},
    process::{Command, Stdio},
    sync::mpsc::{Receiver, Sender},
    thread::{self, JoinHandle},
};

use eframe::egui;
use tray_icon::TrayIconBuilder;

fn main() -> Result<(), eframe::Error> {
    // let icon = load_icon(std::path::Path::new(path));
    let icon_path = concat!(env!("CARGO_MANIFEST_DIR"), "/src/resources/tray_icon-red.png");
    let icon = load_icon(std::path::Path::new(icon_path));

    // Since egui uses winit under the hood and doesn't use gtk on Linux, and we need gtk for
    // the tray icon to show up, we need to spawn a thread
    // where we initialize gtk and create the tray_icon
    #[cfg(target_os = "linux")]
    std::thread::spawn(|| {
        use tray_icon::menu::Menu;

        gtk::init().unwrap();
        let _tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(Menu::new()))
            .with_icon(icon)
            .build()
            .unwrap();

        gtk::main();
    });

    #[cfg(not(target_os = "linux"))]
    let mut _tray_icon = Rc::new(RefCell::new(None));
    #[cfg(not(target_os = "linux"))]
    let tray_c = _tray_icon.clone();

    eframe::run_native(
        "My egui App",
        eframe::NativeOptions::default(),
        Box::new(move |_cc| {
            #[cfg(not(target_os = "linux"))]
            {
                tray_c.replace(TrayIconBuilder::new().with_icon(icon).build().ok());
            }
            Ok(Box::<MyApp>::default())
        }),
    )
}

struct MyApp {
    toggle_show: bool,
    running: Option<JoinHandle<()>>,
    log: Vec<String>,
    output_rx: Option<Receiver<String>>,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            toggle_show: false,
            running: None,
            log: Vec::new(),
            output_rx: None,
        }
    }
}

impl MyApp {
    pub fn start_task(&mut self) {
        // 1) Create the channel
        let (tx, rx): (Sender<String>, Receiver<String>) = std::sync::mpsc::channel();
        self.output_rx = Some(rx);

        // 2) Spawn the process with stdout piped
        let mut cmd = if cfg!(target_os = "windows") {
            let mut c = Command::new("cmd");
            c.args(["/C", "../target/debug/cli.exe"]);
            c
        } else {
            let mut c = Command::new("sh");
            c.args(["-c", "../target/debug/cli"]);
            c
        };
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

        let mut child = cmd.spawn().expect("failed to spawn process");
        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();

        // 3) Read stdout on a new thread
        thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                if let Ok(l) = line {
                    println!("OUT: {}", l);
                    tx.send(l).ok();
                }
            }
            // also drain stderr if you want
            let err_reader = BufReader::new(stderr);
            for line in err_reader.lines() {
                if let Ok(l) = line {
                    tx.send(format!("ERR: {}", l)).ok();
                    tx.send(format!(
                        "Current dir: {}",
                        std::env::current_dir().unwrap().display()
                    ))
                    .ok();
                }
            }
        });

        // 4) Save the JoinHandle so we know it’s running
        self.running = Some(thread::spawn(move || {
            // Optionally wait on child here if you want to know when it exits…
            child.wait().ok();
        }));
    }

    pub fn stop_task(&mut self) {
        if let Some(handle) = self.running.take() {
            handle.join().unwrap();
            self.log.push("Task stopped".to_string());
        } else {
            self.log.push("No task to stop".to_string());
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 0) Drain any new lines from the child process:
        if let Some(rx) = &self.output_rx {
            // Try to pull out all pending messages
            while let Ok(line) = rx.try_recv() {
                self.log.push(line);
            }
            // If you want egui to redraw immediately when new log arrives:
            if !self.log.is_empty() {
                ctx.request_repaint();
            }
        }

        // 1) Tray icon events (as before)…
        use tray_icon::TrayIconEvent;
        if let Ok(event) = TrayIconEvent::receiver().try_recv() {
            match event {
                TrayIconEvent::Click { button_state, .. }
                    if button_state == tray_icon::MouseButtonState::Down =>
                {
                    self.toggle_show = !self.toggle_show;
                }
                _ => {}
            }
        }
        if self.toggle_show {
            ctx.send_viewport_cmd(egui::viewport::ViewportCommand::Visible(true));
        } else {
            ctx.send_viewport_cmd(egui::viewport::ViewportCommand::Visible(false));
        }

        // 2) SidePanel with Start/Stop buttons (unchanged)…
        egui::SidePanel::left("Status")
            .resizable(false)
            .default_width(200.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Status: ");
                    ui.label(if self.running.is_some() {
                        "Running"
                    } else {
                        "Stopped"
                    });
                });
                if self.running.is_some() {
                    if ui.button("Stop").clicked() {
                        self.stop_task();
                    }
                } else if ui.button("Start").clicked() {
                    self.start_task();
                }
            });

        // 3) CentralPanel to display the log:
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Live Log");
            ui.separator();
            // wrap in a scrollable area so it doesn't grow infinitely tall
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .stick_to_bottom(true) // keep scrolling as new lines come in
                .show(ui, |ui| {
                    for line in &self.log {
                        ui.label(line);
                    }
                });
        });
    }
}

fn load_icon(path: &std::path::Path) -> tray_icon::Icon {
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::open(path)
            .expect(&format!(
                "Failed to open icon path {}, current path {}",
                path.display(),
                std::env::current_dir().unwrap().display()
            ))
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    tray_icon::Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("Failed to open icon")
}
