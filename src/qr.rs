use image::{ImageBuffer, Luma, Rgba, imageops::FilterType};
use qrcode::QrCode;

use crate::Args;

pub fn generate_qr(link: &str, args: &Args) -> Result<String, String> {
    if link.trim().is_empty()
    {
        return Err("Le lien ne peut pas être vide.".to_string());
    }

    let code=QrCode::new(link.as_bytes())
        .map_err(|e| format!("QR code invalide à partir du lien: {e}"))?;

    let fg=parse_hex_color(&args.fg).map_err(|e| format!("Couleur de premier plan invalide: {e}"))?;
    let bg=parse_hex_color(&args.bg).map_err(|e| format!("Couleur de fond invalide: {e}"))?;

    let qr_matrix=code
        .render::<Luma<u8>>()
        .min_dimensions(args.size, args.size)
        // Ajoute une bordure (quiet zone) pour améliorer la lisibilité et faciliter le scan.
        .quiet_zone(true)
        .build();

    let background=load_background(args, qr_matrix.width(), qr_matrix.height())?;
    let fg_color=Rgba([fg.0, fg.1, fg.2, 255]);
    let fallback_bg=Rgba([bg.0, bg.1, bg.2, 255]);
    let mut colored: ImageBuffer<Rgba<u8>, Vec<u8>>=
        ImageBuffer::new(qr_matrix.width(), qr_matrix.height());

    // Applique les couleurs personnalisées sur le QR code.
    for (x, y, pixel) in qr_matrix.enumerate_pixels() {
        let val=pixel.0[0];
        let color=if val==0
        {
            fg_color
        }
        else if let Some(bg_img)=&background
        {
            *bg_img.get_pixel(x, y)
        }
        else
        {
            fallback_bg
        };
        colored.put_pixel(x, y, color);
    }

    colored
        .save(&args.output)
        .map_err(|e| format!("Impossible de sauvegarder le fichier: {e}"))?;

    Ok(args.output.clone())
}

fn load_background(
    args: &Args,
    width: u32,
    height: u32,
) -> Result<Option<ImageBuffer<Rgba<u8>, Vec<u8>>>, String>
{
    let Some(path)=args.bg_image.as_ref().filter(|p| !p.trim().is_empty()) else {
        return Ok(None);
    };

    let img=image::open(path)
        .map_err(|e| format!("Impossible de charger l'image de fond \"{path}\": {e}"))?
        .to_rgba8();

    let resized=image::imageops::resize(&img,width,height,FilterType::Triangle);

    Ok(Some(resized))
}

fn parse_hex_color(input: &str) -> Result<(u8, u8, u8), &'static str>
{
    let trimmed=input.trim().trim_start_matches('#');
    if trimmed.len()!= 6
    {
        return Err("attendu format hex sur 6 caractères, ex: #aabbcc");
    }

    let r=u8::from_str_radix(&trimmed[0..2], 16).map_err(|_| "valeur rouge invalide")?;
    let g=u8::from_str_radix(&trimmed[2..4], 16).map_err(|_| "valeur verte invalide")?;
    let b=u8::from_str_radix(&trimmed[4..6], 16).map_err(|_| "valeur bleue invalide")?;
    Ok((r,g,b))
}
