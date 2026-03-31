mod gui;
mod qr;

use clap::Parser;
use std::io::{self, Write};

use crate::gui::launch_gui;
use crate::qr::generate_qr;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about = "Générateur de QR code personnalisable", long_about = None)]
pub struct Args {
    ///Lien à encoder (sinon l'app demandera via stdin)
    link: Option<String>,

    ///Fichier de sortie (svg)
    #[arg(short, long, default_value = "qrcode.svg")]
    output: String,

    ///Taille minimale (px) du QR code
    #[arg(short, long, default_value_t = 256)]
    size: u32,

    ///Couleur de premier plan (hex, ex: #000000)
    #[arg(long, default_value = "#000000")]
    fg: String,

    /// Couleur de fond (hex, ex: #ffffff)
    #[arg(long, default_value = "#ffffff")]
    bg: String,

    /// Image à utiliser comme fond (optionnel)
    #[arg(long, value_name = "PATH")]
    bg_image: Option<String>,

    /// Lancer l'interface graphique plutôt que la CLI
    #[arg(long, default_value_t = false)]
    gui: bool,
}

fn main() {
    let args = Args::parse();

    if args.gui {
        match launch_gui(args.clone()) {
            Ok(()) => return,
            Err(err) => {
                eprintln!("Impossible de démarrer l'interface graphique: {err}");
                eprintln!("Basculement vers le mode CLI interactif...");
            }
        }
    }

    let link = args.link.clone().unwrap_or_else(prompt_for_link);

    match generate_qr(&link, &args) {
        Ok(result) => println!("QR code enregistré dans: {}", result.path),
        Err(err) => {
            eprintln!("Impossible de générer le QR code: {err}");
            std::process::exit(1);
        }
    }
}

fn prompt_for_link() -> String {
    print!("Entrez le lien à encoder: ");
    io::stdout().flush().ok();

    let mut input = String::new();
    if let Err(err) = io::stdin().read_line(&mut input) {
        eprintln!("Erreur lors de la lecture de l'entrée: {err}");
        std::process::exit(1);
    }
    input.trim().to_owned()
}
