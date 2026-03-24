use crate::{qr::generate_qr, Args};
use eframe::egui::{self, ColorImage, FontId, RichText};
use eframe::egui::epaint::Shadow;

pub fn launch_gui(args: Args) -> Result<(), String> {
    if is_headless() {
        return Err(
            "Aucun serveur d'affichage détecté (DISPLAY/WAYLAND_DISPLAY manquantes). Lancez sans --gui ou configurez un environnement graphique.".to_string()
        );
    }

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "QRust code",
        native_options,
        Box::new(move |cc| Box::new(QrGuiApp::new(cc, args.clone()))),
    )
    .map_err(|e| e.to_string())
}

struct QrGuiApp {
    link: String,
    output: String,
    size: u32,
    fg: String,
    bg: String,
    bg_image: String,
    status: Option<StatusMessage>,
    qr_texture: Option<egui::TextureHandle>,
}

struct StatusMessage {
    text: String,
    is_error: bool,
}

impl QrGuiApp {
    fn new(cc: &eframe::CreationContext<'_>, args: Args) -> Self {
        configure_style(&cc.egui_ctx);
        Self {
            link: args.link.unwrap_or_default(),
            output: args.output,
            size: args.size,
            fg: args.fg,
            bg: args.bg,
            bg_image: args.bg_image.unwrap_or_default(),
            status: None,
            qr_texture: None,
        }
    }
}

impl eframe::App for QrGuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let backdrop =
            egui::Frame::none().fill(egui::Color32::from_rgba_premultiplied(245, 248, 255, 255));
        egui::CentralPanel::default()
            .frame(backdrop)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(12.0);
                    ui.label(
                        egui::RichText::new("Générateur de QR code")
                            .size(24.0)
                            .strong(),
                    );
                    ui.add_space(12.0);
                });

                let glass_shadow = Shadow {
                    offset: egui::vec2(0.0, 8.0),
                    blur: 24.0,
                    spread: 0.0,
                    color: egui::Color32::from_rgba_premultiplied(120, 140, 160, 35),
                };
                let glass_frame = egui::Frame::none()
                    .fill(egui::Color32::from_rgba_premultiplied(255, 255, 255, 190))
                    .rounding(egui::Rounding::same(18.0))
                    .stroke(egui::Stroke::new(
                        1.0,
                        egui::Color32::from_rgba_premultiplied(255, 255, 255, 140),
                    ))
                    .inner_margin(egui::Margin::symmetric(18.0, 16.0))
                    .shadow(glass_shadow);

                ui.vertical_centered(|ui| {
                    ui.set_width(ui.available_width().min(620.0));
                    glass_frame.show(ui, |ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(10.0, 10.0);

                        ui.label(
                            egui::RichText::new("Paramètres")
                                .font(FontId::proportional(18.0))
                                .strong(),
                        );
                        ui.add_space(4.0);

                        ui.label("Lien à encoder");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.link)
                                .hint_text("https://exemple.com"),
                        );

                        ui.label("Fichier de sortie (png)");
                        ui.text_edit_singleline(&mut self.output);

                        ui.add(egui::Slider::new(&mut self.size, 64..=2048).text("Taille (px)"));

                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.label("Couleur de premier plan (#rrggbb)");
                                ui.text_edit_singleline(&mut self.fg);
                            });
                            ui.vertical(|ui| {
                                ui.label("Couleur de fond (#rrggbb)");
                                ui.text_edit_singleline(&mut self.bg);
                            });
                        });

                        ui.label("Image de fond (optionnelle)");
                        ui.text_edit_singleline(&mut self.bg_image);

                        ui.add_space(6.0);
                        if ui
                            .add_sized(
                                egui::vec2(ui.available_width(), 38.0),
                                egui::Button::new(
                                    RichText::new("Générer le QR code")
                                        .color(egui::Color32::WHITE),
                                )
                                    .rounding(10.0)
                                    .fill(egui::Color32::from_rgb(54, 123, 245))
                                    .stroke(egui::Stroke::NONE),
                            )
                            .clicked()
                        {
                            self.handle_generate(ctx);
                        }

                        if let Some(status) = &self.status {
                            let color = if status.is_error {
                                egui::Color32::from_rgb(225, 85, 85)
                            } else {
                                egui::Color32::from_rgb(30, 140, 90)
                            };
                            ui.colored_label(color, &status.text);
                        }
                    });

                    if let Some(texture) = &self.qr_texture {
                        ui.add_space(14.0);
                        let mut size = texture.size_vec2();
                        let max_edge = 260.0;
                        if size.x > max_edge || size.y > max_edge {
                            let scale = (max_edge / size.x).min(max_edge / size.y);
                            size *= scale;
                        }
                        let image_shadow = Shadow {
                            offset: egui::vec2(0.0, 6.0),
                            blur: 18.0,
                            spread: 0.0,
                            color: egui::Color32::from_rgba_premultiplied(120, 140, 160, 35),
                        };
                        let image_frame = egui::Frame::none()
                            .fill(egui::Color32::from_rgba_premultiplied(255, 255, 255, 200))
                            .rounding(egui::Rounding::same(16.0))
                            .stroke(egui::Stroke::new(
                                1.0,
                                egui::Color32::from_rgba_premultiplied(255, 255, 255, 140),
                            ))
                            .shadow(image_shadow);

                        image_frame.show(ui, |ui| {
                            ui.set_width(size.x);
                            let image = egui::Image::new(texture).fit_to_exact_size(size);
                            ui.add(image);
                        });
                    }
                });
            });
    }
}

impl QrGuiApp {
    fn handle_generate(&mut self, ctx: &egui::Context) {
        let args = Args {
            link: Some(self.link.clone()),
            output: self.output.clone(),
            size: self.size,
            fg: self.fg.clone(),
            bg: self.bg.clone(),
            bg_image: if self.bg_image.trim().is_empty() {
                None
            } else {
                Some(self.bg_image.clone())
            },
            gui: true,
        };

        match generate_qr(&self.link, &args) {
            Ok(path) => {
                self.status = Some(StatusMessage {
                    text: format!("QR code enregistré dans {path}"),
                    is_error: false,
                });
                self.qr_texture = match load_texture_from_file(ctx, &path) {
                    Ok(texture) => Some(texture),
                    Err(err) => {
                        self.status = Some(StatusMessage {
                            text: err,
                            is_error: true,
                        });
                        None
                    }
                };
            }
            Err(err) => {
                self.status = Some(StatusMessage {
                    text: err,
                    is_error: true,
                });
                self.qr_texture = None;
            }
        }
    }
}

fn load_texture_from_file(ctx: &egui::Context, path: &str) -> Result<egui::TextureHandle, String> {
    let image =
        image::open(path).map_err(|e| format!("QR code généré mais aperçu indisponible: {e}"))?;
    let rgba = image.to_rgba8();
    let size = [rgba.width() as usize, rgba.height() as usize];
    let color_image = ColorImage::from_rgba_unmultiplied(size, rgba.as_raw());
    Ok(ctx.load_texture(
        format!("qr_preview_{}", path),
        color_image,
        egui::TextureOptions::LINEAR,
    ))
}

fn configure_style(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(12.0, 12.0);
    style.spacing.button_padding = egui::vec2(16.0, 10.0);
    style.visuals = egui::Visuals::light();
    style.visuals.window_rounding = egui::Rounding::same(18.0);
    style.visuals.widgets.inactive.rounding = egui::Rounding::same(12.0);
    style.visuals.widgets.hovered.rounding = egui::Rounding::same(12.0);
    style.visuals.widgets.active.rounding = egui::Rounding::same(12.0);
    style.visuals.panel_fill = egui::Color32::from_rgba_premultiplied(245, 248, 255, 255);
    ctx.set_style(style);
}

fn is_headless() -> bool {
    #[cfg(target_os = "linux")]
    {
        std::env::var_os("DISPLAY").is_none() && std::env::var_os("WAYLAND_DISPLAY").is_none()
    }
    #[cfg(target_os = "macos")]
    {
        false
    }
    #[cfg(target_os = "windows")]
    {
        false
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        true
    }
}
