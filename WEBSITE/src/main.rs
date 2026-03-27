use yew::prelude::*;

#[function_component(App)]
fn app() -> Html {
    html! {
        <div class="page">
            <div class="container">
                <nav class="nav">
                    <div class="nav-inner">
                        <a class="brand" href="#top">
                            <span class="brand-mark">{"QR"}</span>
                            <span>{"QRust Code"}</span>
                        </a>
                        <div class="nav-links">
                            <a href="#project">{"Projet"}</a>
                            <a href="#generator">{"Générateur"}</a>
                            <a href="#downloads">{"Téléchargements"}</a>
                        </div>
                        <div class="nav-actions">
                            <a class="pill" href="#downloads">{"Téléchargements"}</a>
                            <a class="button" href="#generator">{"Voir le générateur"}</a>
                        </div>
                    </div>
                </nav>

                <section class="hero shell section" id="top">
                    <div class="hero-copy">
                        <div>
                            <span class="eyebrow">{"Projet S4 · EPITA "}</span>
                            <h1>{"QRust Code — Générateur de QR codes en Rust"}</h1>
                            <p>
                                {"Projet de conception d’un générateur de QR codes en Rust, accompagné d’un site de présentation permettant d’exposer le projet, son interface et ses documents de soutenance."}
                            </p>
                            <div class="hero-actions">
                                <a class="button" href="#generator">{"Voir le générateur"}</a>
                                <a class="button secondary" href="#project">{"Découvrir le projet"}</a>
                            </div>
                        </div>

                        <div class="stats">
                            <div class="stat">
                                <strong>{"Rapide"}</strong>
                                <span>{"génération rapide du QR code"}</span>
                            </div>
                            <div class="stat">
                                <strong>{"4"}</strong>
                                <span>{"étudiants participant au projet"}</span>
                            </div>
                            <div class="stat">
                                <strong>{"Rust"}</strong>
                                <span>{"langage utilisée pour le générateur"}</span>
                            </div>
                        </div>
                    </div>

                    <div class="hero-demo" id="generator">
                        <div class="demo-surface">
                            <div
                                style="
                                    width: 100%;
                                    height: 100%;
                                    min-height: 520px;
                                    border-radius: 28px;
                                    display: grid;
                                    place-items: center;
                                    color: #6f7680;
                                    font-size: 22px;
                                    text-align: center;
                                    padding: 24px;
                                    background: linear-gradient(180deg, rgba(255,255,255,0.95), rgba(239,240,242,0.9));
                                    border: 1px dashed #cfd3d8;
                                "
                            >
                                {"Placeholder PNG"}
                            </div>
                        </div>
                    </div>
                </section>

                <section class="proof-bar section shell" id="project">
                    <div class="proof-items">
                        <div class="proof-item">
                            <div class="icon">{"⚡"}</div>
                            <div>
                                <strong>{"Rapidité"}</strong>
                                <span>{"Une génération efficace et une interface réactive."}</span>
                            </div>
                        </div>
                        <div class="proof-item">
                            <div class="icon">{"🧩"}</div>
                            <div>
                                <strong>{"Interface Optimise"}</strong>
                                <span>{"Une interface simple et minimaliste."}</span>
                            </div>
                        </div>
                        <div class="proof-item">
                            <div class="icon">{"🎨"}</div>
                            <div>
                                <strong>{"Personnalisation"}</strong>
                                <span>{"Choix des couleurs et styles pour adapter le QR code."}</span>
                            </div>
                        </div>
                        <div class="proof-item">
                            <div class="icon">{"🎓"}</div>
                            <div>
                                <strong>{"Projet académique"}</strong>
                                <span>{"Projet S4 de EPITA."}</span>
                            </div>
                        </div>
                    </div>
                </section>

                <section class="section">
                    <h2 class="section-title">{"Présentation du projet"}</h2>
                    <div class="features-3">
                        <article class="usage-card">
                            <strong>{"Historique"}</strong>
                            <p>
                                {"QRust Code est un projet dont l’objectif est de développer un générateur de QR codes en Rust, tout en créant un site web clair et moderne pour présenter le travail."}
                            </p>
                        </article>
                        <article class="usage-card">
                            <strong>{"Équipe"}</strong>
                            <p>
                                {"Projet réalisé par 4 étudiants : Quentin Gaven, William Richard-Chabaud, Ophélien Razia et Noa Lazzaroto. Chacun a travailler sur des aspects différents. "}
                            </p>
                        </article>
                        <article class="usage-card">
                            <strong>{"Chronologie"}</strong>
                            <p>
                                {"Le travail s’est organisé en plusieurs étapes : recherche sur le standard QR code, conception du générateur en Rust, création de l’interface utilisateur, préparation du site de présentation, puis finalisation des documents de soutenance."}
                            </p>
                        </article>
                    </div>
                </section>

                <section class="vision section">
                    <h2>{"Un site clair pour présenter le projet"}</h2>
                    <p>
                        {"Le site regroupe la présentation du générateur de QR codes, du projet en lui-même, des membres de l’équipe et des documents utiles pour la soutenance."}
                    </p>
                    <div class="hero-actions" style="justify-content:center;">
                        <a class="button" href="#downloads">{"Voir les téléchargements"}</a>
                    </div>
                </section>

                <section class="section">
                    <h2 class="section-title">{"Trois étapes vers le générateur"}</h2>
                    <div class="steps">
                        <article class="card">
                            <div class="step-no">{"[1]"}</div>
                            <strong>{"Comprendre le QR code"}</strong>
                            <p>{"Analyse du standard, étude de la structure des modules et compréhension des contraintes techniques avant l’implémentation."}</p>
                        </article>
                        <article class="card">
                            <div class="step-no">{"[2]"}</div>
                            <strong>{"Développer le générateur"}</strong>
                            <p>{"Implémentation du moteur en Rust, structuration de la logique et préparation d’une démonstration cohérente du fonctionnement du projet."}</p>
                        </article>
                        <article class="card dark">
                            <div class="step-no">{"[3]"}</div>
                            <strong>{"Créer l’interface utilisateur"}</strong>
                            <p>{"Conception d’une interface claire pour présenter le générateur, montrer son apparence et exposer le projet de manière lisible."}</p>
                        </article>
                        <article class="card">
                            <div class="step-no">{"[+]"}</div>
                            <strong>{"Perspectives d’évolution"}</strong>
                            <p>{"Ajout d’exports supplémentaires, amélioration du générateur, intégration de nouvelles options visuelles et enrichissement de la partie site web."}</p>
                        </article>
                    </div>
                </section>

                <section class="section" id="downloads">
                    <h2 class="section-title">{"Téléchargements et documents"}</h2>
                    <div
                        class="features-3"
                        style="grid-template-columns: repeat(2, 1fr);"
                    >
                        <article class="usage-card">
                            <strong>{"Générateur QR code"}</strong>
                            <p>{"Télécharger le projet complet contenant le générateur, le site et les éléments utiles à la démonstration."}</p>
                            <div class="hero-actions">
                                <a class="button" href="#">{"Télécharger le projet complet"}</a>
                            </div>
                        </article>
                        <article class="usage-card">
                            <strong>{"Rapport et soutenance"}</strong>
                            <p>{"Accéder au rapport du projet ainsi qu’au plan de la première soutenance."}</p>
                            <div class="hero-actions">
                                <a class="button secondary" href="#">{"Voir le rapport"}</a>
                                <a class="button secondary" href="#">{"Voir le plan de soutenance"}</a>
                            </div>
                        </article>
                    </div>
                </section>

                <footer>
                    <div class="brand">
                        <span class="brand-mark">{"QR"}</span>
                        <span>{"QRust Code"}</span>
                    </div>
                    <div class="footer-links">
                        <a href="#project">{"Projet"}</a>
                        <a href="#downloads">{"Téléchargements"}</a>
                    </div>
                    <div>{"Projet S4 EPITA — site de présentation du projet académique"}</div>
                </footer>
            </div>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
