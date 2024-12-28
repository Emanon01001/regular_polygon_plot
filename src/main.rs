use eframe::egui::{
    self, ColorImage, Slider, TextureOptions, Sense, PointerButton,
};
use eframe::CreationContext;
use eframe::{egui::CentralPanel, App, Frame, NativeOptions};
use image::{ImageBuffer, Rgba, RgbaImage};
use imageproc::drawing::{draw_hollow_circle_mut, draw_line_segment_mut};
use rusttype::{Font, Scale, point, PositionedGlyph};
use std::f64::consts::PI;

/// ---------------------------------------------
/// 多角形の頂点 (x, y, 角度[度]) を生成する関数
/// ---------------------------------------------
fn generate_polygon_points(
    n_sides: usize,
    diameter: f64,   // 外接円の直径
    offset_deg: f64, // ポリゴンを回転させる角度(度数法)
) -> Vec<(f64, f64, f64)> {
    let radius = diameter / 2.0;
    let offset_rad = offset_deg.to_radians();
    let mut points = Vec::new();

    for i in 0..n_sides {
        let base_theta = 2.0 * PI * (i as f64) / (n_sides as f64);
        let theta = base_theta + offset_rad;

        let x = radius * theta.cos();
        let y = radius * theta.sin();

        let mut theta_deg = theta.to_degrees() % 360.0;
        if theta_deg < 0.0 {
            theta_deg += 360.0;
        }

        points.push((x, y, theta_deg));
    }

    points
}

/// ---------------------------------------------
/// RustType を使って文字を描画する関数
/// ---------------------------------------------
fn draw_text(
    img: &mut RgbaImage,
    text: &str,
    x: i32,
    y: i32,
    scale: Scale,
    font: &Font,
    color: [u8; 4],
) {
    let v_metrics = font.v_metrics(scale);
    let glyphs: Vec<PositionedGlyph> = font
        .layout(text, scale, point(0.0, v_metrics.ascent))
        .collect();

    for glyph in glyphs {
        if let Some(bb) = glyph.pixel_bounding_box() {
            glyph.draw(|gx, gy, gv| {
                let px = x + bb.min.x + gx as i32;
                let py = y + bb.min.y + gy as i32;

                if px >= 0 && px < img.width() as i32 && py >= 0 && py < img.height() as i32 {
                    let dst = img.get_pixel_mut(px as u32, py as u32);
                    let alpha = (gv * 255.0) as u8;
                    let inv_alpha = 255 - alpha;

                    let dst_rgba = dst.0;
                    let src_rgba = color;

                    dst.0[0] = ((src_rgba[0] as u16 * alpha as u16
                              + dst_rgba[0] as u16 * inv_alpha as u16) / 255) as u8;
                    dst.0[1] = ((src_rgba[1] as u16 * alpha as u16
                              + dst_rgba[1] as u16 * inv_alpha as u16) / 255) as u8;
                    dst.0[2] = ((src_rgba[2] as u16 * alpha as u16
                              + dst_rgba[2] as u16 * inv_alpha as u16) / 255) as u8;
                    dst.0[3] = 255;
                }
            });
        }
    }
}

/// ---------------------------------------------
/// eguiアプリ用の構造体
/// ---------------------------------------------
struct PolygonApp {
    n_sides: usize,
    diameter: f64,
    offset_deg: f64,

    zoom: f32,                          // 画像のズーム比率
    image_texture: Option<egui::TextureHandle>,
}

impl PolygonApp {
    fn new(cc: &CreationContext<'_>) -> Self {
        // フォント設定 (省略可能)
        setup_custom_fonts(&cc.egui_ctx);

        Self {
            n_sides: 8,
            diameter: 700.0,   // 半径350
            offset_deg: 22.5,  // 4辺が軸と平行になる回転
            zoom: 1.0,
            image_texture: None,
        }
    }
}

fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // 例: プロジェクト内の meiryo.ttc を読み込む
    fonts.font_data.insert(
        "meiryo".to_owned(),
        egui::FontData::from_static(include_bytes!("./meiryo.ttc")).into(),
    );

    if let Some(proportional) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
        proportional.insert(0, "meiryo".to_owned());
    }
    if let Some(monospace) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
        monospace.insert(0, "meiryo".to_owned());
    }

    ctx.set_fonts(fonts);
}

impl App for PolygonApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        // メインUI
        CentralPanel::default().show(ctx, |ui| {
            ui.heading("正多角形プロットツール (Click on image to zoom)");
            ui.separator();

            // 多角形パラメータ
            ui.add(Slider::new(&mut self.n_sides, 3..=20).text("n_sides (>=3)"));
            ui.add(Slider::new(&mut self.diameter, 100.0..=5000.0).text("Diameter"));
            ui.add(Slider::new(&mut self.offset_deg, 0.0..=45.0).text("Offset (deg)"));

            // ズーム倍率
            ui.label(format!(
                "Current Zoom: {:.2}x (Left-click=ZoomIn, Right-click=ZoomOut)",
                self.zoom
            ));

            // [Generate] ボタン
            if ui.button("Generate").clicked() {
                // diameter の値に応じて画像サイズを動的に決定
                let rgba = create_image(self.n_sides, self.diameter, self.offset_deg);

                // ColorImage に変換
                let size = [rgba.width() as usize, rgba.height() as usize];
                let mut rgba_data = Vec::with_capacity(size[0] * size[1] * 4);
                for (_, _, pixel) in rgba.enumerate_pixels() {
                    rgba_data.extend_from_slice(&pixel.0);
                }

                let color_image = ColorImage::from_rgba_unmultiplied(size, &rgba_data);

                // テクスチャとしてアップロード (最近傍補間)
                let texture_handle = ctx.load_texture(
                    "polygon_image",
                    color_image,
                    TextureOptions::NEAREST,
                );
                self.image_texture = Some(texture_handle);
            }

            ui.separator();

            // 生成画像を表示
            if let Some(img_texture) = &self.image_texture {
                let size_vec = img_texture.size_vec2();
                let scaled_size = size_vec * self.zoom;

                egui::ScrollArea::both()
                    .max_width(ui.available_width())
                    .max_height(ui.available_height())
                    .show(ui, |ui| {
                        let image_widget = egui::Image::new((img_texture.id(), scaled_size))
                            .sense(Sense::click());
                        let response = ui.add(image_widget);

                        if response.hovered() {
                            // 左クリック = ズームイン
                            if response.clicked_by(PointerButton::Primary) {
                                self.zoom *= 1.1;
                                self.zoom = self.zoom.clamp(0.1, 10.0);
                            }
                            // 右クリック = ズームアウト
                            if response.clicked_by(PointerButton::Secondary) {
                                self.zoom /= 1.1;
                                self.zoom = self.zoom.clamp(0.1, 10.0);
                            }
                        }
                    });
            }
        });
    }
}

/// ---------------------------------------------
/// 画像生成
/// ---------------------------------------------
fn create_image(n_sides: usize, diameter: f64, offset_deg: f64) -> RgbaImage {
    // diameter が大きくなっても切れないよう、画像サイズを動的に拡大
    //   半径 = diameter/2 で ±radius 程度の範囲を使うので、
    //   + 200ピクセル程度の余白を足しておく
    let needed_size = (diameter as u32 + 200).max(1000); // 少なくとも 1000x1000
    let (img_width, img_height) = (needed_size, needed_size);

    // 背景色
    let bg_color = [220u8, 220u8, 220u8, 255u8];
    let mut img: RgbaImage = ImageBuffer::from_fn(img_width, img_height, |_x, _y| Rgba(bg_color));

    let points = generate_polygon_points(n_sides, diameter, offset_deg);

    // 画像中心
    let cx = img_width as f64 / 2.0;
    let cy = img_height as f64 / 2.0;

    // 外接円
    let radius_i = (diameter / 2.0).round() as i32;
    draw_hollow_circle_mut(&mut img, (cx as i32, cy as i32), radius_i, Rgba([0, 0, 255, 255]));

    // 多角形
    let mut img_points = Vec::new();
    for &(x, y, _) in &points {
        // y軸反転はしない(後で可視化するだけ)
        let px = (cx + x).round() as i32;
        let py = (cy - y).round() as i32;
        img_points.push((px, py));
    }
    for i in 0..n_sides {
        let j = (i + 1) % n_sides;
        draw_line_segment_mut(
            &mut img,
            (img_points[i].0 as f32, img_points[i].1 as f32),
            (img_points[j].0 as f32, img_points[j].1 as f32),
            Rgba([0, 0, 0, 255]),
        );
    }

    // 頂点に赤丸+テキスト
    let font_data = include_bytes!("meiryo.ttc"); // 適宜変える
    let font = Font::try_from_bytes(font_data as &[u8]).unwrap();
    let scale = Scale { x: 18.0, y: 18.0 };
    let red = Rgba([255, 0, 0, 255]);

    let r = 3;
    for (i, &(px, py)) in img_points.iter().enumerate() {
        // 塗りつぶしの赤丸
        for dy in -r..=r {
            for dx in -r..=r {
                if dx * dx + dy * dy <= r * r {
                    let xx = px + dx;
                    let yy = py + dy;
                    if xx >= 0 && xx < img_width as i32 && yy >= 0 && yy < img_height as i32 {
                        img.put_pixel(xx as u32, yy as u32, red);
                    }
                }
            }
        }
        // 座標と角度ラベル
        let (orig_x, orig_y, angle_deg) = points[i];
        let text_str = format!("({:.0}, {:.0}) / {:.1}°", orig_x, orig_y, angle_deg);
        draw_text(&mut img, &text_str, px + 6, py - 12, scale, &font, [0, 0, 0, 255]);
    }

    img
}

/// ---------------------------------------------
/// main
/// ---------------------------------------------
fn main() {
    let native_opts = NativeOptions::default();
    let _ = eframe::run_native(
        "Octagon & Circle Generator (Click on image to Zoom)",
        native_opts,
        Box::new(|cc| Ok(Box::new(PolygonApp::new(cc)))),
    );
}