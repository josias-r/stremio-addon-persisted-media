use axum::{extract::State, response::Html};
use crate::routes::AppState;

pub async fn home(State(state): State<AppState>) -> Html<String> {
    let manifest_url = format!("{}/manifest.json", state.config.public_url);
    
    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Mini Media Server Addon</title>
    <style>
        :root {{
            --primary: #6366f1;
            --primary-hover: #4f46e5;
            --bg: #0f172a;
            --card-bg: #1e293b;
            --text: #f8fafc;
            --text-muted: #94a3b8;
            --border: #334155;
        }}
        
        body {{ 
            font-family: 'Inter', system-ui, -apple-system, sans-serif; 
            line-height: 1.6; 
            margin: 0; 
            padding: 0; 
            background-color: var(--bg);
            color: var(--text);
            min-height: 100vh;
            display: flex;
            flex-direction: column;
            align-items: center;
        }}
        
        .container {{
            max-width: 800px;
            width: 100%;
            padding: 3rem 2rem;
            box-sizing: border-box;
        }}
        
        header {{
            text-align: center;
            margin-bottom: 4rem;
            animation: fadeIn 0.8s ease-out;
        }}
        
        h1 {{ 
            font-size: 3rem;
            background: linear-gradient(to right, #818cf8, #c084fc);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
            margin-bottom: 0.5rem;
        }}
        
        p.subtitle {{
            font-size: 1.2rem;
            color: var(--text-muted);
        }}
        
        .section {{
            margin-bottom: 3rem;
            animation: slideUp 0.8s ease-out both;
        }}
        
        .section:nth-child(3) {{ animation-delay: 0.2s; }}
        
        h2 {{
            font-size: 1.8rem;
            border-bottom: 2px solid var(--border);
            padding-bottom: 0.5rem;
            margin-bottom: 1.5rem;
        }}
        
        .manifest-card {{ 
            background: var(--card-bg); 
            padding: 2rem; 
            border-radius: 12px; 
            border: 1px solid var(--border); 
            text-align: center;
            box-shadow: 0 10px 25px -5px rgba(0, 0, 0, 0.3);
            transition: transform 0.3s ease, box-shadow 0.3s ease;
        }}
        
        .manifest-card:hover {{
            transform: translateY(-5px);
            box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.4);
        }}
        
        .manifest-url {{ 
            background: rgba(0, 0, 0, 0.3);
            padding: 1rem;
            border-radius: 8px;
            font-family: 'JetBrains Mono', monospace;
            font-size: 1.1em;
            color: #a78bfa;
            word-break: break-all;
            margin: 1.5rem 0;
            border: 1px solid rgba(255, 255, 255, 0.1);
        }}
        
        .btn {{
            display: inline-block;
            background: var(--primary);
            color: white;
            padding: 0.8rem 1.5rem;
            border-radius: 8px;
            font-weight: 600;
            text-decoration: none;
            transition: background 0.2s, transform 0.2s;
            cursor: pointer;
            border: none;
            font-size: 1rem;
        }}
        
        .btn:hover {{
            background: var(--primary-hover);
            transform: scale(1.05);
        }}
        
        .client-list {{ 
            display: grid; 
            grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
            gap: 1.5rem; 
        }}
        
        .client-card {{ 
            background: var(--card-bg);
            padding: 1.5rem; 
            border: 1px solid var(--border); 
            border-radius: 12px; 
            transition: all 0.3s ease;
            display: flex;
            flex-direction: column;
        }}
        
        .client-card:hover {{ 
            border-color: var(--primary);
            transform: translateY(-5px);
            box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.2);
        }}
        
        .client-card h3 {{ 
            margin-top: 0; 
            margin-bottom: 0.5rem;
        }}
        
        .client-card p {{
            color: var(--text-muted);
            flex-grow: 1;
            margin-bottom: 1.5rem;
        }}
        
        .client-card a {{ 
            color: var(--text); 
            text-decoration: none;
            font-weight: 600;
            display: inline-flex;
            align-items: center;
            gap: 0.5rem;
            transition: color 0.2s;
        }}
        
        .client-card a:hover {{ 
            color: var(--primary); 
        }}
        
        @keyframes fadeIn {{
            from {{ opacity: 0; }}
            to {{ opacity: 1; }}
        }}
        
        @keyframes slideUp {{
            from {{ opacity: 0; transform: translateY(20px); }}
            to {{ opacity: 1; transform: translateY(0); }}
        }}
    </style>
</head>
<body>
    <div class="container">
        <header>
            <h1>Mini Media Server</h1>
            <p class="subtitle">Your self-hosted media server addon is running successfully</p>
        </header>
        
        <div class="section">
            <h2>Installation</h2>
            <div class="manifest-card">
                <p>To use this addon, copy the manifest URL below and paste it into the addons settings of your stremio client.</p>
                
                <div class="manifest-url" id="manifestUrl">
                    {manifest_url}
                </div>
                
                <button class="btn" onclick="copyToClipboard()">Copy URL to Clipboard</button>
            </div>
        </div>

        <div class="section">
            <h2>Compatible Clients</h2>
            <p style="margin-bottom: 1.5rem; color: var(--text-muted);">This addon is compatible with <strong>any</strong> Stremio-compatible client. Here are just a few popular examples you can use to enjoy your media:</p>
            
            <div class="client-list">
                <div class="client-card">
                    <h3>Stremio</h3>
                    <p>The official Stremio client. Available for Windows, macOS, Linux, Android, and Android TV.</p>
                    <a href="https://www.stremio.com/downloads" target="_blank">
                        Download Stremio <span>→</span>
                    </a>
                </div>
                
                <div class="client-card">
                    <h3>Fusion - MediaCenter</h3>
                    <p>A popular third-party Stremio-compatible media center tailored for iOS and Apple TV users.</p>
                    <a href="https://apps.apple.com/us/app/fusion-media-center/id6759285919" target="_blank">
                        View on App Store <span>→</span>
                    </a>
                </div>
                
                <div class="client-card">
                    <h3>Harbor</h3>
                    <p>A customizable, third-party open-source desktop client built for the Stremio ecosystem.</p>
                    <a href="https://github.com/harborstremio/harbor" target="_blank">
                        View on GitHub <span>→</span>
                    </a>
                </div>
            </div>
        </div>
    </div>
    
    <script>
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
    </script>
</body>
</html>"#,
        manifest_url = manifest_url
    );

    Html(html)
}
