use crate::{
    Args,
    qr::{generate_qr, generate_qr_preview},
};
use eframe::egui::epaint::{Hsva, Shadow};
use eframe::egui::{self, ColorImage, FontId, RichText};
use rfd::FileDialog;
use std::f32::consts::TAU;
use std::path::Path;
use std::time::{Duration, Instant};

const PREVIEW_MAX_SIZE: u32 = 720;

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
    fg_hsva: Hsva,
    bg_hsva: Hsva,
    bg_image: String,
    status: Option<StatusMessage>,
    qr_texture: Option<egui::TextureHandle>,
    preview_dirty: bool,
    last_preview_change: Option<Instant>,
    lock_bg_with_image: bool,
}

struct StatusMessage {
    text: String,
    is_error: bool,
}

impl QrGuiApp {
    fn new(cc: &eframe::CreationContext<'_>, args: Args) -> Self {
        configure_style(&cc.egui_ctx);
        let fg_color = hex_to_color32(&args.fg, egui::Color32::BLACK);
        let bg_color = hex_to_color32(&args.bg, egui::Color32::WHITE);
        let fg_hsva: Hsva = fg_color.into();
        let bg_hsva: Hsva = bg_color.into();
        let initial_fg: egui::Color32 = fg_hsva.into();
        let initial_bg: egui::Color32 = bg_hsva.into();
        let mut app = Self {
            link: args.link.unwrap_or_default(),
            output: args.output,
            size: args.size,
            fg: color_to_hex(initial_fg),
            bg: color_to_hex(initial_bg),
            fg_hsva,
            bg_hsva,
            bg_image: args.bg_image.unwrap_or_default(),
            status: None,
            qr_texture: None,
            preview_dirty: false,
            last_preview_change: None,
            lock_bg_with_image: true,
        };

        if !app.link.trim().is_empty() {
            app.refresh_preview(&cc.egui_ctx);
        }

        app
    }
}

impl eframe::App for QrGuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let backdrop =
            egui::Frame::none().fill(egui::Color32::from_rgba_premultiplied(245, 248, 255, 255));
        egui::CentralPanel::default()
            .frame(backdrop)
            .show(ctx, |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    ui.add_space(12.0);
                    ui.label(
                        egui::RichText::new("Générateur de QR code")
                            .size(24.0)
                            .strong(),
                    );
                    ui.add_space(12.0);

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

                    let available_width = ui.available_width();
                    let container_width = (available_width * 0.82).clamp(520.0, 1080.0);

                    glass_frame.show(ui, |ui| {
                        ui.set_width(container_width);
                        ui.spacing_mut().item_spacing = egui::vec2(10.0, 10.0);
                        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                            ui.label(
                                egui::RichText::new("Paramètres")
                                    .font(FontId::proportional(18.0))
                                    .strong(),
                            );
                            ui.add_space(4.0);

                            egui::Grid::new("settings_grid")
                                .num_columns(2)
                                .min_col_width(160.0)
                                .spacing(egui::vec2(12.0, 14.0))
                                .show(ui, |ui| {
                                    ui.label("Lien à encoder");
                                    let link_response = ui.add(
                                        egui::TextEdit::singleline(&mut self.link)
                                            .hint_text("https://exemple.com")
                                            .desired_width(f32::INFINITY),
                                    );
                                    if link_response.changed() {
                                        self.mark_preview_dirty(ctx);
                                    }
                                    ui.end_row();

                                    ui.label("Fichier de sortie (svg)");
                                    ui.text_edit_singleline(&mut self.output);
                                    ui.end_row();

                                    ui.label("Taille (px)");
                                    let size_slider = ui.add(
                                        egui::Slider::new(&mut self.size, 64..=2048)
                                            .text("Taille (px)"),
                                    );
                                    if size_slider.changed() {
                                        self.mark_preview_dirty(ctx);
                                    }
                                    ui.end_row();

                                    ui.label("Couleurs");
                                    ui.horizontal(|ui| {
                                        ui.vertical(|ui| {
                                            ui.label("Premier plan");
                                            let fg_changed =
                                                color_wheel_picker(ui, &mut self.fg_hsva);
                                            if fg_changed {
                                                let color: egui::Color32 = self.fg_hsva.into();
                                                self.fg = color_to_hex(color);
                                            }
                                            if fg_changed {
                                                self.mark_preview_dirty(ctx);
                                            }
                                            ui.label(format!("Hex: {}", self.fg));
                                        });
                                        ui.add_space(10.0);
                                        let bg_locked =
                                            self.lock_bg_with_image && !self.bg_image.trim().is_empty();
                                        ui.add_enabled_ui(!bg_locked, |ui| {
                                            ui.vertical(|ui| {
                                                ui.label("Arrière-plan");
                                                let bg_changed =
                                                    color_wheel_picker(ui, &mut self.bg_hsva);
                                                if bg_changed {
                                                    let color: egui::Color32 = self.bg_hsva.into();
                                                    self.bg = color_to_hex(color);
                                                }
                                                if bg_changed {
                                                    self.mark_preview_dirty(ctx);
                                                }
                                                ui.label(format!("Hex: {}", self.bg));
                                            });
                                        });
                                        if bg_locked {
                                            ui.label(
                                                egui::RichText::new("Couleur verrouillée (image active)")
                                                    .italics()
                                                    .color(egui::Color32::from_gray(90)),
                                            );
                                        }
                                    });
                                    ui.end_row();
                                });

                            ui.add_space(12.0);
                            ui.horizontal_wrapped(|ui| {
                                ui.label("Image de fond (optionnelle)");
                                let select_clicked = ui
                                    .add(
                                        egui::Button::new(
                                            RichText::new("Choisir une image...")
                                                .color(egui::Color32::from_rgb(40, 90, 180)),
                                        )
                                        .rounding(8.0)
                                        .fill(egui::Color32::from_rgb(235, 242, 255))
                                        .stroke(egui::Stroke::new(
                                            1.0,
                                            egui::Color32::from_rgb(190, 210, 240),
                                        )),
                                    )
                                    .clicked();
                                if select_clicked {
                                    if let Some(path) = FileDialog::new()
                                        .add_filter("Images", &["png", "jpg", "jpeg", "bmp", "gif"])
                                        .pick_file()
                                    {
                                        let chosen = path.display().to_string();
                                        if chosen != self.bg_image {
                                            self.bg_image = chosen;
                                            self.mark_preview_dirty(ctx);
                                        }
                                    }
                                }

                                ui.add_enabled_ui(!self.bg_image.trim().is_empty(), |ui| {
                                    let remove_button = egui::Button::new(
                                        RichText::new("Retirer l'image")
                                            .color(egui::Color32::from_rgb(190, 55, 55)),
                                    )
                                    .rounding(8.0)
                                    .fill(egui::Color32::from_rgb(255, 232, 232))
                                    .stroke(egui::Stroke::new(
                                        1.0,
                                        egui::Color32::from_rgb(245, 190, 190),
                                    ));
                                    if ui.add(remove_button).clicked() {
                                        self.clear_background_image(ctx);
                                    }
                                });

                                if !self.bg_image.trim().is_empty() {
                                    ui.label(
                                        RichText::new(selected_image_label(&self.bg_image))
                                            .color(egui::Color32::from_gray(70)),
                                    );
                                } else {
                                    ui.label(
                                        RichText::new("Aucune image sélectionnée")
                                            .color(egui::Color32::from_gray(120)),
                                    );
                                }

                                let checkbox = ui.checkbox(
                                    &mut self.lock_bg_with_image,
                                    "Verrouiller la couleur si une image est utilisée",
                                );
                                if checkbox.changed() {
                                    self.mark_preview_dirty(ctx);
                                }
                            });

                            ui.add_space(6.0);
                            let generate_clicked = ui
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
                                .clicked();
                            if generate_clicked {
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
                    });

                    if let Some(texture) = &self.qr_texture {
                        ui.add_space(14.0);
                        let mut size = texture.size_vec2();
                        let max_edge = (ui.available_width().min(ui.available_height()) * 0.55)
                            .clamp(260.0, 540.0);
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

        self.process_preview_refresh(ctx);
    }
}

impl QrGuiApp {
    fn mark_preview_dirty(&mut self, ctx: &egui::Context) {
        self.preview_dirty = true;
        self.last_preview_change = Some(Instant::now());
        ctx.request_repaint_after(Duration::from_millis(50));
    }

    fn clear_background_image(&mut self, ctx: &egui::Context) {
        if self.bg_image.trim().is_empty() {
            return;
        }
        self.bg_image.clear();
        self.mark_preview_dirty(ctx);
    }

    fn process_preview_refresh(&mut self, ctx: &egui::Context) {
        if !self.preview_dirty {
            return;
        }

        let elapsed = self
            .last_preview_change
            .map(|t| t.elapsed())
            .unwrap_or_else(|| Duration::from_millis(0));

        if elapsed >= Duration::from_millis(140) {
            self.preview_dirty = false;
            self.refresh_preview(ctx);
        } else {
            ctx.request_repaint_after(Duration::from_millis(60));
        }
    }

    fn refresh_preview(&mut self, ctx: &egui::Context) {
        if self.link.trim().is_empty() {
            self.qr_texture = None;
            return;
        }

        let args = Args {
            link: Some(self.link.clone()),
            output: self.output.clone(),
            size: self.size.min(PREVIEW_MAX_SIZE),
            fg: self.fg.clone(),
            bg: self.bg.clone(),
            bg_image: if self.bg_image.trim().is_empty() {
                None
            } else {
                Some(self.bg_image.clone())
            },
            gui: true,
        };

        match generate_qr_preview(&self.link, &args) {
            Ok(result) => {
                self.qr_texture = Some(texture_from_buffer(ctx, result.preview, "live_preview"));
            }
            Err(_) => {
                self.qr_texture = None;
            }
        }
    }

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
            Ok(result) => {
                let path = result.path;
                self.status = Some(StatusMessage {
                    text: format!("QR code enregistré dans {path}"),
                    is_error: false,
                });
                self.qr_texture = Some(texture_from_buffer(ctx, result.preview, &path));
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

fn selected_image_label(path_str: &str) -> String {
    let trimmed = path_str.trim();
    let name = Path::new(trimmed)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(trimmed);
    format!("Sélectionnée: {name}")
}

fn color_wheel_picker(ui: &mut egui::Ui, hsva: &mut Hsva) -> bool {
    let mut changed = false;
    let size = 150.0;
    let (rect, response) =
        ui.allocate_at_least(egui::vec2(size, size), egui::Sense::click_and_drag());
    let radius = rect.width().min(rect.height()) * 0.5;
    let center = rect.center();
    let wheel_value = hsva.v.max(0.35); // keep the wheel vivid even for very dark selections

    if ui.is_rect_visible(rect) {
        let mut mesh = egui::Mesh::default();
        let segments = 72;
        let rings = 28;
        for ring in 0..rings {
            let r0 = radius * (ring as f32 / rings as f32);
            let r1 = radius * ((ring + 1) as f32 / rings as f32);
            for i in 0..segments {
                let a0 = i as f32 / segments as f32 * TAU;
                let a1 = (i + 1) as f32 / segments as f32 * TAU;
                let h0 = a0 / TAU;
                let h1 = a1 / TAU;
                let s0 = r0 / radius;
                let s1 = r1 / radius;

                let c00: egui::Color32 = Hsva {
                    h: h0,
                    s: s0,
                    v: wheel_value,
                    a: 1.0,
                }
                .into();
                let c01: egui::Color32 = Hsva {
                    h: h0,
                    s: s1,
                    v: wheel_value,
                    a: 1.0,
                }
                .into();
                let c10: egui::Color32 = Hsva {
                    h: h1,
                    s: s0,
                    v: wheel_value,
                    a: 1.0,
                }
                .into();
                let c11: egui::Color32 = Hsva {
                    h: h1,
                    s: s1,
                    v: wheel_value,
                    a: 1.0,
                }
                .into();

                let p00 = center + egui::vec2(a0.cos(), a0.sin()) * r0;
                let p01 = center + egui::vec2(a0.cos(), a0.sin()) * r1;
                let p10 = center + egui::vec2(a1.cos(), a1.sin()) * r0;
                let p11 = center + egui::vec2(a1.cos(), a1.sin()) * r1;

                let idx = mesh.vertices.len() as u32;
                mesh.colored_vertex(p00, c00);
                mesh.colored_vertex(p01, c01);
                mesh.colored_vertex(p10, c10);
                mesh.colored_vertex(p11, c11);

                mesh.add_triangle(idx, idx + 1, idx + 2);
                mesh.add_triangle(idx + 1, idx + 2, idx + 3);
            }
        }
        ui.painter().add(egui::Shape::mesh(mesh));
        ui.painter().circle_stroke(
            center,
            radius,
            egui::Stroke::new(1.0, egui::Color32::from_gray(160)),
        );

        let handle_pos =
            center + egui::vec2((hsva.h * TAU).cos(), (hsva.h * TAU).sin()) * (hsva.s * radius);
        let handle_color: egui::Color32 = (*hsva).into();
        ui.painter().circle_filled(handle_pos, 7.0, handle_color);
        ui.painter().circle_stroke(
            handle_pos,
            9.0,
            egui::Stroke::new(2.0, egui::Color32::WHITE),
        );
    }

    if let Some(pointer) = response.interact_pointer_pos() {
        let delta = pointer - center;
        if delta.length_sq() > 0.0 {
            let angle = delta.angle();
            hsva.h = (angle / TAU).rem_euclid(1.0);
            hsva.s = (delta.length() / radius).min(1.0);
            if hsva.v < 0.5 {
                hsva.v = 0.85;
            }
            hsva.a = 1.0;
            changed = true;
        }
    }

    ui.add_space(8.0);
    let slider = ui.add(
        egui::Slider::new(&mut hsva.v, 0.0..=1.0)
            .text("Luminosité")
            .show_value(false),
    );
    changed |= slider.changed();

    changed
}

fn texture_from_buffer(
    ctx: &egui::Context,
    image: image::ImageBuffer<image::Rgba<u8>, Vec<u8>>,
    id: &str,
) -> egui::TextureHandle {
    let size = [image.width() as usize, image.height() as usize];
    let data = image.into_raw();
    let color_image = ColorImage::from_rgba_unmultiplied(size, &data);
    ctx.load_texture(
        format!("qr_preview_{}", id),
        color_image,
        egui::TextureOptions::LINEAR,
    )
}

fn hex_to_color32(input: &str, fallback: egui::Color32) -> egui::Color32 {
    let trimmed = input.trim().trim_start_matches('#');
    if trimmed.len() == 6 {
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&trimmed[0..2], 16),
            u8::from_str_radix(&trimmed[2..4], 16),
            u8::from_str_radix(&trimmed[4..6], 16),
        ) {
            return egui::Color32::from_rgb(r, g, b);
        }
    }
    fallback
}

fn color_to_hex(color: egui::Color32) -> String {
    format!("#{:02x}{:02x}{:02x}", color.r(), color.g(), color.b())
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
