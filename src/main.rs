#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use schild3_updater::{
    config::{self, Settings},
    self_update,
    service::{self, UpdatePlan},
};
use std::sync::mpsc::{self, Receiver};

const WEBSITE: &str = "https://optima-computer.de/";
const BRAND: egui::Color32 = egui::Color32::from_rgb(0, 53, 152);
const INK: egui::Color32 = egui::Color32::from_rgb(30, 37, 47);
const MUTED: egui::Color32 = egui::Color32::from_rgb(93, 103, 116);
const BORDER: egui::Color32 = egui::Color32::from_rgb(213, 220, 229);

fn configure_appearance(context: &egui::Context) {
    context.set_theme(egui::Theme::Light);
    let mut fonts = egui::FontDefinitions::default();
    if let Some(windows) = std::env::var_os("WINDIR") {
        if let Ok(bytes) =
            std::fs::read(std::path::PathBuf::from(windows).join("Fonts/segoeui.ttf"))
        {
            fonts
                .font_data
                .insert("windows-ui".into(), egui::FontData::from_owned(bytes));
            fonts
                .families
                .get_mut(&egui::FontFamily::Proportional)
                .unwrap()
                .insert(0, "windows-ui".into());
        }
    }
    context.set_fonts(fonts);
    let mut style = (*context.style()).clone();
    style
        .text_styles
        .insert(egui::TextStyle::Body, egui::FontId::proportional(16.0));
    style
        .text_styles
        .insert(egui::TextStyle::Button, egui::FontId::proportional(16.0));
    style
        .text_styles
        .insert(egui::TextStyle::Heading, egui::FontId::proportional(21.0));
    style
        .text_styles
        .insert(egui::TextStyle::Small, egui::FontId::proportional(13.0));
    style
        .text_styles
        .insert(egui::TextStyle::Monospace, egui::FontId::monospace(15.0));
    style.spacing.item_spacing = egui::vec2(12.0, 12.0);
    style.spacing.button_padding = egui::vec2(16.0, 10.0);
    style.spacing.interact_size = egui::vec2(44.0, 44.0);
    style.visuals = egui::Visuals::light();
    style.visuals.override_text_color = Some(INK);
    style.visuals.panel_fill = egui::Color32::WHITE;
    style.visuals.extreme_bg_color = egui::Color32::WHITE;
    style.visuals.selection.bg_fill = BRAND;
    style.visuals.selection.stroke = egui::Stroke::new(1.0_f32, egui::Color32::WHITE);
    style.visuals.hyperlink_color = BRAND;
    for widget in [
        &mut style.visuals.widgets.inactive,
        &mut style.visuals.widgets.hovered,
        &mut style.visuals.widgets.active,
    ] {
        widget.rounding = egui::Rounding::same(5.0);
        widget.bg_stroke = egui::Stroke::new(1.0_f32, BORDER);
    }
    style.visuals.widgets.inactive.weak_bg_fill = egui::Color32::from_rgb(243, 246, 249);
    style.visuals.widgets.hovered.weak_bg_fill = egui::Color32::from_rgb(229, 236, 247);
    context.set_style(style);
    egui_extras::install_image_loaders(context);
}

fn symbol_button(
    ui: &mut egui::Ui,
    source: egui::ImageSource<'static>,
    tooltip: &str,
) -> egui::Response {
    ui.add_sized(
        [44.0, 44.0],
        egui::Button::image(egui::Image::new(source).fit_to_exact_size(egui::vec2(20.0, 20.0))),
    )
    .on_hover_text(tooltip)
}

fn app_icon() -> egui::IconData {
    let image = image::load_from_memory(include_bytes!(concat!(env!("OUT_DIR"), "/app-icon.png")))
        .expect("Eingebettetes Programm-Icon ist ungültig")
        .into_rgba8();
    egui::IconData {
        width: image.width(),
        height: image.height(),
        rgba: image.into_raw(),
    }
}

enum Event {
    Progress(String),
    UpdaterChecked(Option<self_update::Update>),
    UpdaterCheckFailed(String),
    UpdaterInstalled(std::path::PathBuf),
    Plan(UpdatePlan),
    ConfirmUpdate(UpdatePlan),
    Done(String),
    Failed(String),
}

enum Operation {
    CheckUpdater,
    InstallUpdater(self_update::Update),
    Check { notify: bool },
    CheckAndInstall,
    Install(UpdatePlan),
}

struct UpdaterApp {
    settings: Settings,
    receiver: Option<Receiver<Event>>,
    plan: Option<UpdatePlan>,
    status: String,
    error: Option<String>,
    log: Vec<String>,
    busy: bool,
    logo: egui::TextureHandle,
    show_about: bool,
    startup_pending: bool,
    confirmation: Option<UpdatePlan>,
    updater_startup_pending: bool,
    updater_confirmation: Option<self_update::Update>,
    updater_status: String,
    restart: Option<std::path::PathBuf>,
    restarting: bool,
}

impl UpdaterApp {
    fn new(context: &eframe::CreationContext<'_>) -> Self {
        configure_appearance(&context.egui_ctx);
        let image = image::load_from_memory(include_bytes!("../assets/optima-logo-white.png"))
            .expect("Eingebettetes Firmenlogo ist ungültig")
            .into_rgba8();
        let logo = context.egui_ctx.load_texture(
            "optima-logo",
            egui::ColorImage::from_rgba_unmultiplied(
                [image.width() as usize, image.height() as usize],
                image.as_raw(),
            ),
            egui::TextureOptions::LINEAR,
        );
        let (settings, error) = match config::load() {
            Ok(settings) => (settings, None),
            Err(error) => (Settings::default(), Some(format!("{error:#}"))),
        };
        let startup_pending = error.is_none();
        Self {
            settings,
            receiver: None,
            plan: None,
            status: "Bereit".into(),
            error,
            log: Vec::new(),
            busy: false,
            logo,
            show_about: false,
            startup_pending,
            confirmation: None,
            updater_startup_pending: true,
            updater_confirmation: None,
            updater_status: "Noch nicht geprüft".into(),
            restart: None,
            restarting: false,
        }
    }

    fn record(&mut self, message: String) {
        self.log.push(message);
        if self.log.len() > 200 {
            self.log.remove(0);
        }
    }

    fn check_on_startup(&mut self, context: &egui::Context) {
        if self.restarting
            || self.busy
            || self.confirmation.is_some()
            || self.updater_confirmation.is_some()
        {
            return;
        }
        if std::mem::take(&mut self.updater_startup_pending) {
            self.begin(context, Operation::CheckUpdater);
            return;
        }
        if !std::mem::take(&mut self.startup_pending) {
            return;
        }
        if self.settings.server_url.trim().is_empty() || self.settings.client_dir.trim().is_empty()
        {
            self.status = "Bitte Serveradresse und Clientordner konfigurieren.".into();
            return;
        }
        self.begin(context, Operation::Check { notify: true });
    }

    fn updater_confirmation_response(&mut self, accepted: bool) -> Option<self_update::Update> {
        let update = self.updater_confirmation.take();
        if accepted {
            update
        } else {
            self.status = "Updater-Update abgebrochen. Die EXE bleibt unverändert.".into();
            self.record(self.status.clone());
            None
        }
    }

    fn confirmation_response(&mut self, accepted: bool) -> Option<UpdatePlan> {
        let plan = self.confirmation.take();
        if accepted {
            plan
        } else {
            self.status = "Update abgebrochen. Es wurden keine Clientdateien verändert.".into();
            self.record(self.status.clone());
            None
        }
    }

    fn begin(&mut self, context: &egui::Context, operation: Operation) {
        if self.busy {
            return;
        }
        if matches!(
            operation,
            Operation::Check { notify: false } | Operation::CheckAndInstall
        ) {
            if let Err(error) = config::save(&self.settings) {
                self.error = Some(format!("{error:#}"));
                return;
            }
        }
        self.busy = true;
        if !matches!(operation, Operation::CheckUpdater) {
            self.error = None;
        }
        if !matches!(
            operation,
            Operation::CheckUpdater | Operation::InstallUpdater(_)
        ) {
            self.plan = None;
        }
        self.confirmation = None;
        let checking_updater = matches!(operation, Operation::CheckUpdater);
        self.status = if checking_updater {
            "Neue Updater-Version wird gesucht ...".into()
        } else {
            "Vorgang läuft ...".into()
        };
        let settings = self.settings.clone();
        let (sender, receiver) = mpsc::channel();
        self.receiver = Some(receiver);
        let context = context.clone();
        std::thread::spawn(move || {
            let emit = |event| {
                let _ = sender.send(event);
                context.request_repaint();
            };
            let progress = |message| emit(Event::Progress(message));
            let result =
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> anyhow::Result<()> {
                    let (plan, install, notify) = match operation {
                        Operation::CheckUpdater => {
                            emit(Event::UpdaterChecked(self_update::check()?));
                            return Ok(());
                        }
                        Operation::InstallUpdater(update) => {
                            emit(Event::UpdaterInstalled(self_update::install(
                                &update, &progress,
                            )?));
                            return Ok(());
                        }
                        Operation::Check { notify } => {
                            (service::check(&settings, &progress)?, false, notify)
                        }
                        Operation::CheckAndInstall => {
                            (service::check(&settings, &progress)?, true, false)
                        }
                        Operation::Install(plan) => (plan, true, false),
                    };
                    emit(Event::Plan(plan.clone()));
                    if plan.installed == plan.target {
                        emit(Event::Done(format!(
                            "Client {} passt zu SVWS {}. Kein Update erforderlich.",
                            plan.installed, plan.server
                        )));
                    } else if install {
                        let backup = service::update(&plan, &progress)?;
                        let mut completed = plan.clone();
                        completed.installed = plan.target.clone();
                        emit(Event::Plan(completed));
                        emit(Event::Done(format!(
                            "Update auf {} abgeschlossen. Sicherung: {}",
                            plan.target,
                            backup.display()
                        )));
                    } else {
                        if notify {
                            emit(Event::ConfirmUpdate(plan.clone()));
                        }
                        emit(Event::Done(format!(
                            "Update verfügbar: {} → {} für SVWS {}.",
                            plan.installed, plan.target, plan.server
                        )));
                    }
                    Ok(())
                }));
            match result {
                Ok(Err(error)) if checking_updater => emit(Event::UpdaterCheckFailed(format!("{error:#}"))),
                Ok(Err(error)) => emit(Event::Failed(format!("{error:#}"))),
                Err(_) => emit(Event::Failed("Unerwarteter Programmfehler. Bitte das Protokoll und gegebenenfalls den Sicherungsordner prüfen.".into())),
                Ok(Ok(())) => (),
            }
        });
    }

    fn poll(&mut self) {
        let events: Vec<Event> = self
            .receiver
            .as_ref()
            .map(|receiver| receiver.try_iter().collect())
            .unwrap_or_default();
        for event in events {
            match event {
                Event::UpdaterChecked(update) => {
                    self.updater_status = match &update {
                        Some(update) => format!("Updater {} verfügbar", update.version),
                        None => format!("Updater {} ist aktuell", env!("CARGO_PKG_VERSION")),
                    };
                    self.status = self.updater_status.clone();
                    self.record(self.updater_status.clone());
                    self.updater_confirmation = update;
                    self.busy = false;
                }
                Event::UpdaterCheckFailed(message) => {
                    self.updater_status = format!("Updater-Prüfung nicht möglich: {message}");
                    self.status = self.updater_status.clone();
                    self.record(self.updater_status.clone());
                    self.busy = false;
                }
                Event::UpdaterInstalled(executable) => {
                    self.restart = Some(executable);
                    self.busy = false;
                }
                Event::Progress(message) => {
                    self.status = message.clone();
                    self.record(message);
                }
                Event::Plan(plan) => self.plan = Some(plan),
                Event::ConfirmUpdate(plan) => self.confirmation = Some(plan),
                Event::Done(message) => {
                    self.status = message.clone();
                    self.record(message);
                    self.busy = false;
                }
                Event::Failed(message) => {
                    self.confirmation = None;
                    self.record(message.clone());
                    self.error = Some(message);
                    self.status = "Vorgang fehlgeschlagen".into();
                    self.busy = false;
                }
            }
        }
        if !self.busy {
            self.receiver = None;
        }
    }
}

impl eframe::App for UpdaterApp {
    fn update(&mut self, context: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll();
        if self.restarting {
            context.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        if let Some(executable) = self.restart.take() {
            match std::process::Command::new(executable).spawn() {
                Ok(_) => {
                    self.restarting = true;
                    context.send_viewport_cmd(egui::ViewportCommand::Close);
                    return;
                }
                Err(error) => {
                    self.startup_pending = false;
                    self.status = "Updater installiert. Bitte manuell neu starten.".into();
                    self.error = Some(format!("Updater installiert, Neustart fehlgeschlagen: {error}. Bitte das Programm schließen und erneut starten."));
                    self.record(self.error.clone().unwrap());
                }
            }
        }
        self.check_on_startup(context);
        if self.busy && context.input(|input| input.viewport().close_requested()) {
            context.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.record("Bitte den laufenden Vorgang vor dem Schließen abwarten.".into());
        }
        egui::TopBottomPanel::top("header")
            .frame(
                egui::Frame::none()
                    .fill(BRAND)
                    .inner_margin(egui::Margin::symmetric(28.0, 24.0)),
            )
            .show(context, |ui| {
                ui.horizontal(|ui| {
                    if ui
                        .add(
                            egui::Image::new(&self.logo)
                                .fit_to_exact_size(egui::vec2(183.0, 50.0))
                                .sense(egui::Sense::click()),
                        )
                        .on_hover_text("OPTIMA Computer Service: Webseite öffnen")
                        .clicked()
                    {
                        context.open_url(egui::OpenUrl::new_tab(WEBSITE));
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new("SchILD3 Updater")
                                .size(26.0)
                                .strong()
                                .color(egui::Color32::WHITE),
                        );
                    });
                });
            });
        egui::TopBottomPanel::bottom("footer")
            .frame(
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(243, 246, 249))
                    .inner_margin(egui::Margin::symmetric(28.0, 10.0)),
            )
            .show(context, |ui| {
                ui.horizontal(|ui| {
                    ui.hyperlink_to("optima-computer.de", WEBSITE)
                        .on_hover_text("Webseite im Standardbrowser öffnen");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Info").clicked() {
                            self.show_about = true;
                        }
                        ui.label(
                            egui::RichText::new(concat!("Version ", env!("CARGO_PKG_VERSION")))
                                .small()
                                .color(MUTED),
                        );
                    });
                });
            });
        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(egui::Color32::WHITE)
                    .inner_margin(28.0),
            )
            .show(context, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("workspace")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.add_enabled_ui(
                            !self.busy
                                && self.confirmation.is_none()
                                && self.updater_confirmation.is_none(),
                            |ui| {
                                ui.horizontal(|ui| {
                                    ui.heading("Verbindung und Client");
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            if symbol_button(
                                                ui,
                                                egui::include_image!("../assets/save.svg"),
                                                "Einstellungen speichern",
                                            )
                                            .clicked()
                                            {
                                                match config::save(&self.settings) {
                                                    Ok(()) => {
                                                        self.status =
                                                            "Einstellungen gespeichert".into();
                                                        self.error = None;
                                                    }
                                                    Err(error) => {
                                                        self.error = Some(format!("{error:#}"))
                                                    }
                                                }
                                            }
                                        },
                                    );
                                });
                                ui.label(egui::RichText::new("SVWS-Serveradresse").color(MUTED));
                                let address = ui.add(
                                    egui::TextEdit::singleline(&mut self.settings.server_url)
                                        .hint_text("https://schule.example.org/")
                                        .margin(egui::vec2(12.0, 11.0))
                                        .font(egui::FontId::proportional(16.0))
                                        .desired_width(f32::INFINITY),
                                );
                                ui.label(egui::RichText::new("SchILD3-Clientordner").color(MUTED));
                                let mut changed = address.changed();
                                ui.horizontal(|ui| {
                                    changed |= ui
                                        .add_sized(
                                            [(ui.available_width() - 56.0).max(100.0), 44.0],
                                            egui::TextEdit::singleline(
                                                &mut self.settings.client_dir,
                                            )
                                            .margin(egui::vec2(12.0, 11.0))
                                            .font(egui::FontId::proportional(16.0))
                                            .hint_text(
                                                r"C:\SchILD3 oder \\Server\Freigabe\SchILD3",
                                            ),
                                        )
                                        .changed();
                                    if symbol_button(
                                        ui,
                                        egui::include_image!("../assets/folder-open.svg"),
                                        "Clientordner auswählen",
                                    )
                                    .clicked()
                                    {
                                        if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                            self.settings.client_dir =
                                                path.to_string_lossy().into_owned();
                                            changed = true;
                                        }
                                    }
                                });
                                if changed {
                                    self.plan = None;
                                    self.status = "Einstellungen geändert".into();
                                }
                            },
                        );
                        ui.add_space(8.0);
                        ui.separator();
                        ui.heading("Versionsabgleich");
                        ui.columns(3, |columns| {
                            for (column, (label, value)) in columns.iter_mut().zip([
                                (
                                    "SVWS-Server",
                                    self.plan.as_ref().map(|plan| plan.server.to_string()),
                                ),
                                (
                                    "Installierter Client",
                                    self.plan.as_ref().map(|plan| plan.installed.to_string()),
                                ),
                                (
                                    "Passender Client",
                                    self.plan.as_ref().map(|plan| plan.target.to_string()),
                                ),
                            ]) {
                                column.set_min_height(70.0);
                                column.label(egui::RichText::new(label).size(14.0).color(MUTED));
                                match value {
                                    Some(version) => {
                                        column.label(
                                            egui::RichText::new(version).size(25.0).strong(),
                                        );
                                    }
                                    None => {
                                        column.label(
                                            egui::RichText::new("Nicht geprüft")
                                                .size(16.0)
                                                .color(MUTED),
                                        );
                                    }
                                }
                            }
                        });
                        ui.separator();
                        ui.add_enabled_ui(
                            !self.busy
                                && self.confirmation.is_none()
                                && self.updater_confirmation.is_none(),
                            |ui| {
                                ui.horizontal_wrapped(|ui| {
                                    if ui
                                        .add_sized([132.0, 44.0], egui::Button::new("Nur prüfen"))
                                        .clicked()
                                    {
                                        self.begin(context, Operation::Check { notify: false });
                                    }
                                    if ui
                                        .add_sized(
                                            [260.0, 44.0],
                                            egui::Button::new(
                                                egui::RichText::new("Prüfen und aktualisieren")
                                                    .color(egui::Color32::WHITE),
                                            )
                                            .fill(BRAND),
                                        )
                                        .clicked()
                                    {
                                        self.begin(context, Operation::CheckAndInstall);
                                    }
                                });
                            },
                        );
                        ui.horizontal_wrapped(|ui| {
                            if self.busy {
                                ui.spinner();
                            } else {
                                let color = if self.error.is_some() {
                                    egui::Color32::from_rgb(185, 43, 53)
                                } else {
                                    egui::Color32::from_rgb(20, 125, 92)
                                };
                                let (rect, _) = ui.allocate_exact_size(
                                    egui::vec2(10.0, 20.0),
                                    egui::Sense::hover(),
                                );
                                ui.painter().circle_filled(rect.center(), 4.0, color);
                            }
                            ui.add(egui::Label::new(&self.status).wrap());
                        });
                        if let Some(error) = &self.error {
                            ui.colored_label(egui::Color32::from_rgb(170, 35, 40), error);
                        }
                        ui.separator();
                        ui.horizontal(|ui| {
                            ui.heading("Protokoll");
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if symbol_button(
                                        ui,
                                        egui::include_image!("../assets/copy.svg"),
                                        "Protokoll kopieren",
                                    )
                                    .clicked()
                                    {
                                        context.copy_text(self.log.join("\n"));
                                    }
                                },
                            );
                        });
                        egui::ScrollArea::vertical()
                            .id_salt("log")
                            .min_scrolled_height(100.0)
                            .max_height(190.0)
                            .stick_to_bottom(true)
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                if self.log.is_empty() {
                                    ui.label(egui::RichText::new("Keine Einträge").color(MUTED));
                                }
                                for line in &self.log {
                                    ui.add(
                                        egui::Label::new(egui::RichText::new(line).size(14.0))
                                            .wrap(),
                                    );
                                }
                            });
                    });
            });
        let mut check_updater = false;
        egui::Window::new("Über SchILD3 Updater")
            .open(&mut self.show_about)
            .default_width(540.0)
            .resizable(true)
            .show(context, |ui| {
                ui.heading("OPTIMA Computer Service");
                ui.label(concat!(
                    "SchILD3 Updater · Version ",
                    env!("CARGO_PKG_VERSION")
                ));
                ui.hyperlink_to("optima-computer.de", WEBSITE);
                ui.separator();
                ui.hyperlink_to("GitHub-Releases", self_update::RELEASES_PAGE);
                ui.add(egui::Label::new(&self.updater_status).wrap());
                check_updater = ui
                    .add_enabled(
                        !self.busy
                            && self.confirmation.is_none()
                            && self.updater_confirmation.is_none(),
                        egui::Button::new("Auf Updater-Update prüfen"),
                    )
                    .clicked();
                ui.separator();
                ui.collapsing("Lizenzen der Bedienicons (Lucide)", |ui| {
                    egui::ScrollArea::vertical()
                        .max_height(240.0)
                        .show(ui, |ui| {
                            ui.label(include_str!("../assets/lucide-LICENSE"));
                        });
                });
            });
        if check_updater {
            self.begin(context, Operation::CheckUpdater);
        }
        if let Some(update) = self.updater_confirmation.clone() {
            let mut open = true;
            let mut accepted = false;
            let mut cancelled = false;
            egui::Window::new("Neue Updater-Version verfügbar")
                .id(egui::Id::new("updater_confirmation"))
                .open(&mut open)
                .order(egui::Order::Foreground)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .collapsible(false)
                .resizable(false)
                .default_width(480.0)
                .show(context, |ui| {
                    ui.label(format!("SchILD3 Updater {} → {}", env!("CARGO_PKG_VERSION"), update.version));
                    ui.add(egui::Label::new("Die neue Updater-EXE wird von GitHub geladen. Anschließend startet der Updater neu. Einstellungen und Clientdateien bleiben erhalten.").wrap());
                    ui.separator();
                    ui.horizontal(|ui| {
                        accepted = ui.add_sized([120.0, 44.0], egui::Button::new(egui::RichText::new("OK").color(egui::Color32::WHITE)).fill(BRAND)).clicked();
                        cancelled = ui.add_sized([140.0, 44.0], egui::Button::new("Abbrechen")).clicked();
                    });
                });
            if !open || cancelled || context.input(|input| input.key_pressed(egui::Key::Escape)) {
                self.updater_confirmation_response(false);
                context.request_repaint();
            } else if accepted {
                if let Some(update) = self.updater_confirmation_response(true) {
                    self.begin(context, Operation::InstallUpdater(update));
                }
            }
        }
        if let Some(plan) = self.confirmation.clone() {
            let mut open = true;
            let mut accepted = false;
            let mut cancelled = false;
            egui::Window::new("SchILD3-Update verfügbar")
                .id(egui::Id::new("update_confirmation"))
                .open(&mut open)
                .order(egui::Order::Foreground)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .collapsible(false)
                .resizable(false)
                .default_width(480.0)
                .show(context, |ui| {
                    ui.label("Ein zur Serverversion passendes Client-Update ist verfügbar.");
                    ui.add_space(8.0);
                    egui::Grid::new("confirmation_versions").spacing([24.0, 10.0]).show(ui, |ui| {
                        for (label, version) in [
                            ("SVWS-Server", &plan.server),
                            ("Installierter Client", &plan.installed),
                            ("Neuer Client", &plan.target),
                        ] {
                            ui.label(label);
                            ui.strong(version.to_string());
                            ui.end_row();
                        }
                    });
                    ui.add_space(8.0);
                    ui.add(egui::Label::new("Vor dem Update SchILD3 auf allen betroffenen Arbeitsplätzen schließen.").wrap());
                    ui.separator();
                    ui.horizontal(|ui| {
                        accepted = ui.add_sized([120.0, 44.0], egui::Button::new(egui::RichText::new("OK").color(egui::Color32::WHITE)).fill(BRAND)).clicked();
                        cancelled = ui.add_sized([140.0, 44.0], egui::Button::new("Abbrechen")).clicked();
                    });
                });
            if !open || cancelled || context.input(|input| input.key_pressed(egui::Key::Escape)) {
                self.confirmation_response(false);
            } else if accepted {
                if let Some(plan) = self.confirmation_response(true) {
                    self.begin(context, Operation::Install(plan));
                }
            }
        }
    }
}

fn main() -> eframe::Result<()> {
    let arguments: Vec<String> = std::env::args().collect();
    if arguments.get(1).map(String::as_str) == Some("--check") {
        let result = command_line_check(&arguments);
        if let Err(error) = result {
            eprintln!("{error:#}");
            std::process::exit(1);
        }
        return Ok(());
    }
    eframe::run_native(
        "SchILD3 Updater",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([920.0, 820.0])
                .with_min_inner_size([680.0, 620.0])
                .with_icon(app_icon()),
            ..Default::default()
        },
        Box::new(|context| Ok(Box::new(UpdaterApp::new(context)))),
    )
}

fn command_line_check(arguments: &[String]) -> anyhow::Result<()> {
    anyhow::ensure!(
        arguments.len() == 4,
        "Aufruf: schild3-updater.exe --check https://server/ CLIENTORDNER"
    );
    let settings = Settings {
        server_url: arguments[2].clone(),
        client_dir: arguments[3].clone(),
    };
    let plan = service::check(&settings, &|message| println!("{message}"))?;
    println!(
        "Server: {}; Client: {}; Passender Client: {}",
        plan.server, plan.installed, plan.target
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app() -> (egui::Context, UpdaterApp) {
        let context = egui::Context::default();
        let logo = context.load_texture(
            "test-logo",
            egui::ColorImage::new([1, 1], egui::Color32::WHITE),
            egui::TextureOptions::default(),
        );
        let app = UpdaterApp {
            settings: Settings::default(),
            receiver: None,
            plan: None,
            status: "Bereit".into(),
            error: None,
            log: Vec::new(),
            busy: false,
            logo,
            show_about: false,
            startup_pending: true,
            confirmation: None,
            updater_startup_pending: false,
            updater_confirmation: None,
            updater_status: "Noch nicht geprüft".into(),
            restart: None,
            restarting: false,
        };
        (context, app)
    }

    fn test_plan() -> UpdatePlan {
        UpdatePlan {
            settings: Settings {
                server_url: "https://server.example/".into(),
                client_dir: r"C:\SchILD3".into(),
            },
            installed: semver::Version::new(3, 5, 0),
            server: semver::Version::new(1, 5, 1),
            target: semver::Version::new(3, 6, 0),
            asset: service::Asset {
                name: "Setup_SchILD3_v3.6.0.zip".into(),
                browser_download_url: String::new(),
                digest: None,
            },
        }
    }

    #[test]
    fn startup_without_configuration_does_not_start_or_repeat() {
        let (context, mut app) = test_app();
        app.check_on_startup(&context);
        assert!(!app.startup_pending);
        assert!(!app.busy);
        assert!(app.receiver.is_none());
        app.settings = test_plan().settings;
        app.check_on_startup(&context);
        assert!(!app.busy);
    }

    #[test]
    fn startup_result_waits_for_consent_and_can_be_cancelled() {
        let (_, mut app) = test_app();
        let (sender, receiver) = mpsc::channel();
        app.receiver = Some(receiver);
        app.busy = true;
        sender.send(Event::Plan(test_plan())).unwrap();
        sender.send(Event::ConfirmUpdate(test_plan())).unwrap();
        sender.send(Event::Done("Update verfügbar".into())).unwrap();
        app.poll();
        assert!(app.confirmation.is_some());
        assert!(!app.busy);
        assert!(app.confirmation_response(false).is_none());
        assert!(app.confirmation.is_none());
        assert!(!app.busy);
        assert!(app.receiver.is_none());
    }

    #[test]
    fn accepting_returns_only_the_displayed_update_plan() {
        let (_, mut app) = test_app();
        app.confirmation = Some(test_plan());
        app.settings.server_url = "https://other.example/".into();
        let accepted = app.confirmation_response(true).unwrap();
        assert_eq!(accepted.settings.server_url, "https://server.example/");
        assert_eq!(accepted.target, semver::Version::new(3, 6, 0));
        assert!(app.confirmation_response(true).is_none());
    }

    #[test]
    fn current_client_does_not_request_confirmation() {
        let (_, mut app) = test_app();
        let mut plan = test_plan();
        plan.installed = plan.target.clone();
        let (sender, receiver) = mpsc::channel();
        app.receiver = Some(receiver);
        app.busy = true;
        sender.send(Event::Plan(plan)).unwrap();
        sender
            .send(Event::Done("Kein Update erforderlich".into()))
            .unwrap();
        app.poll();
        assert!(app.confirmation.is_none());
        assert!(!app.busy);
    }

    #[test]
    fn updater_offer_waits_for_consent_before_client_check() {
        let (context, mut app) = test_app();
        let (sender, receiver) = mpsc::channel();
        app.receiver = Some(receiver);
        app.busy = true;
        sender
            .send(Event::UpdaterChecked(Some(self_update::Update {
                version: semver::Version::new(0, 3, 0),
                asset: test_plan().asset,
            })))
            .unwrap();
        app.poll();
        app.check_on_startup(&context);
        assert!(app.startup_pending);
        assert!(!app.busy);
        assert!(app.restart.is_none());
        assert!(app.updater_confirmation_response(false).is_none());
        app.check_on_startup(&context);
        assert!(!app.startup_pending);
        assert!(app.updater_confirmation.is_none());
    }

    #[test]
    fn updater_check_failure_does_not_block_client_startup() {
        let (context, mut app) = test_app();
        let (sender, receiver) = mpsc::channel();
        app.receiver = Some(receiver);
        app.busy = true;
        sender
            .send(Event::UpdaterCheckFailed("GitHub 404".into()))
            .unwrap();
        app.poll();
        app.check_on_startup(&context);
        assert!(!app.busy);
        assert!(!app.startup_pending);
        assert!(app.error.is_none());
        assert!(app.log.iter().any(|message| message.contains("GitHub 404")));
    }

    #[test]
    fn restarting_does_not_begin_another_client_check() {
        let (context, mut app) = test_app();
        app.restarting = true;
        app.settings = test_plan().settings;
        app.check_on_startup(&context);
        assert!(app.startup_pending);
        assert!(!app.busy);
        assert!(app.receiver.is_none());
    }

    #[test]
    fn updater_consent_is_consumed_only_once() {
        let (_, mut app) = test_app();
        app.updater_confirmation = Some(self_update::Update {
            version: semver::Version::new(0, 3, 0),
            asset: test_plan().asset,
        });
        assert_eq!(
            app.updater_confirmation_response(true).unwrap().version,
            semver::Version::new(0, 3, 0)
        );
        assert!(app.updater_confirmation_response(true).is_none());
    }

    #[test]
    #[cfg(windows)]
    fn windows_version_matches_package_version() {
        assert_eq!(
            schild3_updater::version_info::read_version(&std::env::current_exe().unwrap()).unwrap(),
            semver::Version::parse(env!("CARGO_PKG_VERSION")).unwrap(),
        );
    }

    #[test]
    fn typography_survives_initial_theme_selection() {
        let context = egui::Context::default();
        context.set_theme(egui::Theme::Dark);
        configure_appearance(&context);
        context.set_theme(egui::Theme::Light);
        let style = context.style();
        assert_eq!(style.text_styles[&egui::TextStyle::Body].size, 16.0);
        assert_eq!(style.text_styles[&egui::TextStyle::Button].size, 16.0);
        assert_eq!(style.visuals.hyperlink_color, BRAND);
    }

    #[test]
    fn embedded_branding_is_high_resolution_and_valid() {
        let logo =
            image::load_from_memory(include_bytes!("../assets/optima-logo-white.png")).unwrap();
        assert!(logo.width() >= 900 && logo.height() >= 250);
        let icon = app_icon();
        assert_eq!((icon.width, icon.height), (256, 256));
        assert_eq!(icon.rgba.len(), 256 * 256 * 4);
    }
}
