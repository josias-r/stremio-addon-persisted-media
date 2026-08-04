use maud::{html, Markup, DOCTYPE};

pub fn base_layout(title: &str, subtitle: &str, content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="UTF-8";
                meta name="viewport" content="width=device-width, initial-scale=1.0";
                title { (title) }
                style {
                    (maud::PreEscaped(r#"
                    :root {
                        --primary: #6366f1;
                        --primary-hover: #4f46e5;
                        --bg: #0f172a;
                        --card-bg: #1e293b;
                        --text: #f8fafc;
                        --text-muted: #94a3b8;
                        --border: #334155;
                    }

                    *, *::before, *::after {
                        box-sizing: border-box;
                    }
                    
                    body { 
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
                    }
                    
                    .container {
                        max-width: 800px;
                        width: 100%;
                        padding: 3rem 2rem;
                        box-sizing: border-box;
                    }
                    
                    header {
                        text-align: center;
                        margin-bottom: 4rem;
                        animation: fadeIn 0.8s ease-out;
                    }
                    
                    h1 { 
                        font-size: 3rem;
                        background: linear-gradient(to right, #818cf8, #c084fc);
                        -webkit-background-clip: text;
                        -webkit-text-fill-color: transparent;
                        margin-bottom: 0.5rem;
                    }
                    
                    p.subtitle {
                        font-size: 1.2rem;
                        color: var(--text-muted);
                    }
                    
                    .section {
                        margin-bottom: 3rem;
                        animation: slideUp 0.8s ease-out both;
                    }
                    
                    h2 {
                        font-size: 1.8rem;
                        border-bottom: 2px solid var(--border);
                        padding-bottom: 0.5rem;
                        margin-bottom: 1.5rem;
                    }
                    
                    .card { 
                        background: var(--card-bg); 
                        padding: 2rem; 
                        border-radius: 12px; 
                        border: 1px solid var(--border); 
                        box-shadow: 0 10px 25px -5px rgba(0, 0, 0, 0.3);
                    }

                    .btn {
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
                    }
                    
                    .btn:hover {
                        background: var(--primary-hover);
                        transform: scale(1.05);
                    }
                    
                    .btn-danger {
                        background: #ef4444;
                    }

                    .btn-danger:hover {
                        background: #dc2626;
                    }

                    table {
                        width: 100%;
                        border-collapse: collapse;
                        margin-bottom: 2rem;
                        border-radius: 8px;
                        overflow: hidden;
                    }

                    th, td {
                        padding: 1rem;
                        text-align: left;
                        border-bottom: 1px solid var(--border);
                    }

                    th {
                        background-color: rgba(255,255,255,0.05);
                        font-weight: 600;
                        color: var(--text);
                    }

                    td {
                        color: var(--text-muted);
                    }

                    .input-field {
                        padding: 0.8rem; 
                        border-radius: 8px; 
                        border: 1px solid var(--border); 
                        background: var(--bg); 
                        color: var(--text); 
                        margin-bottom: 1rem;
                        font-size: 16px;
                    }
                    
                    @keyframes fadeIn {
                        from { opacity: 0; }
                        to { opacity: 1; }
                    }
                    
                    @keyframes slideUp {
                        from { opacity: 0; transform: translateY(20px); }
                        to { opacity: 1; transform: translateY(0); }
                    }
                    "#))
                }
            }
            body {
                div class="container" {
                    header {
                        h1 { (title) }
                        p class="subtitle" { (subtitle) }
                    }
                    (content)
                }
            }
        }
    }
}
