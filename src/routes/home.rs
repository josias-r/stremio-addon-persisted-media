use axum::{extract::State, response::IntoResponse};
use maud::{html, PreEscaped};
use crate::routes::AppState;
use crate::routes::ui::base_layout;

pub async fn home(State(state): State<AppState>) -> impl IntoResponse {
    let public_url = state.config.public_url.clone();
    
    let content = html! {
        style {
            (PreEscaped(r#"
                .client-list { 
                    display: grid; 
                    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
                    gap: 1.5rem; 
                }
                .client-card { 
                    background: var(--card-bg);
                    padding: 1.5rem; 
                    border: 1px solid var(--border); 
                    border-radius: 12px; 
                    transition: all 0.3s ease;
                    display: flex;
                    flex-direction: column;
                }
                .client-card:hover { 
                    border-color: var(--primary);
                    transform: translateY(-5px);
                    box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.2);
                }
                .client-card h3 { margin-top: 0; margin-bottom: 0.5rem; }
                .client-card p {
                    color: var(--text-muted);
                    flex-grow: 1;
                    margin-bottom: 1.5rem;
                }
                .client-card a { 
                    color: var(--text); 
                    text-decoration: none;
                    font-weight: 600;
                    display: inline-flex;
                    align-items: center;
                    gap: 0.5rem;
                    transition: color 0.2s;
                }
                .client-card a:hover { color: var(--primary); }
                
                .manifest-url { 
                    background: rgba(0, 0, 0, 0.3);
                    padding: 1rem;
                    border-radius: 8px;
                    font-family: 'JetBrains Mono', monospace;
                    font-size: 1.1em;
                    color: #a78bfa;
                    word-break: break-all;
                    margin: 1.5rem 0;
                    border: 1px solid rgba(255, 255, 255, 0.1);
                }
            "#))
        }

        div class="section" {
            h2 { "Installation" }
            div class="card" style="text-align: center; margin-bottom: 2rem;" {
                p { "Paste the API key the admin provided you here:" }
                input type="text" id="apiKeyInput" class="input-field" placeholder="Enter API Key..." style="width: 100%;" oninput="updateManifestUrl()" {}
                
                div class="manifest-url" id="manifestUrl" {
                    (public_url) "/YOUR_API_KEY/manifest.json"
                }
                
                button class="btn" onclick="copyToClipboard()" { "Copy URL to Clipboard" }
            }

            div class="card" {
                h3 style="margin-top: 0; margin-bottom: 1rem; font-size: 1.2rem; color: var(--text);" { "How to connect:" }
                ol style="margin: 0; padding-left: 1.5rem; color: var(--text-muted); line-height: 1.8;" {
                    li { "Get your API key from the admin (it looks something like " code style="background: rgba(255,255,255,0.1); padding: 0.2rem 0.4rem; border-radius: 4px; font-size: 0.9em;" { "550e8400-e29b-41d4-a716-446655440000" } ")." }
                    li { "Paste it in the box above, then press " strong style="color: var(--text);" { "Copy URL to Clipboard" } "." }
                    li { "Download one of the compatible clients below." }
                    li { "Open your client, go to " strong style="color: var(--text);" { "Settings ➔ Addons" } "." }
                    li { "Paste the copied Addon URL (it should end with " code style="background: rgba(255,255,255,0.1); padding: 0.2rem 0.4rem; border-radius: 4px; font-size: 0.9em;" { "/manifest.json" } ") and click install." }
                    li { "Browse movies or shows in the client and stream via the options provided by this addon! 🍿🎉" }
                }
            }
        }

        div class="section" {
            h2 { "Compatible Clients" }
            p style="margin-bottom: 1.5rem; color: var(--text-muted);" {
                "This addon is compatible with " strong { "any" } " Stremio-compatible client. Here are just a few popular examples you can use to enjoy your media:"
            }
            
            div class="client-list" {
                div class="client-card" {
                    h3 { "Stremio" }
                    p { "The official Stremio client. Available for Windows, macOS, Linux, Android, and Android TV." }
                    a href="https://www.stremio.com/downloads" target="_blank" {
                        "Download Stremio " span { "→" }
                    }
                }
                
                div class="client-card" {
                    h3 { "Fusion - Media Center" }
                    p { "A popular third-party Stremio-compatible media center tailored for iOS and Apple TV users." }
                    a href="https://apps.apple.com/us/app/fusion-media-center/id6759285919" target="_blank" {
                        "View on App Store " span { "→" }
                    }
                }
                
                div class="client-card" {
                    h3 { "Harbor" }
                    p { "A customizable, third-party open-source desktop client built for the Stremio ecosystem." }
                    a href="https://github.com/harborstremio/harbor" target="_blank" {
                        "View on GitHub " span { "→" }
                    }
                }
            }
        }
        
        div style="text-align: center; padding: 2rem; font-size: 0.85rem; color: var(--text-muted);" {
            a href="/admin" style="color: inherit; text-decoration: none; opacity: 0.7; transition: opacity 0.2s;" 
               onmouseover="this.style.opacity=1" onmouseout="this.style.opacity=0.7" { "Admin Panel" }
        }

        script {
            (PreEscaped(format!(r#"
            function updateManifestUrl() {{
                const apiKey = document.getElementById('apiKeyInput').value.trim() || 'YOUR_API_KEY';
                document.getElementById('manifestUrl').innerText = `{public_url}/${{apiKey}}/manifest.json`;
            }}

            function copyToClipboard() {{
                const url = document.getElementById('manifestUrl').innerText;
                navigator.clipboard.writeText(url).then(() => {{
                    const btn = document.querySelector('.btn');
                    const originalText = btn.innerText;
                    btn.innerText = 'Copied!';
                    btn.style.background = '#10b981';
                    setTimeout(() => {{
                        btn.innerText = originalText;
                        btn.style.background = 'var(--primary)';
                    }}, 2000);
                }}).catch(err => {{
                    console.error('Failed to copy: ', err);
                }});
            }}
            "#, public_url = public_url)))
        }
    };

    base_layout("Mini Media Server", "Your self-hosted media server addon is running successfully", content)
}
