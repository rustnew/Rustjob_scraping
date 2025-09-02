use reqwest::Client;
use scraper::{Html, Selector};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🚀 Démarrage du scraping de RustJobs.dev...");
    
    let client = Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let response = client.get("https://rustjobs.dev/").send().await?;
    let html = response.text().await?;
    let document = Html::parse_document(&html);

    println!("📊 Taille du HTML: {} caractères", html.len());

    // Sauvegarder le HTML pour inspection
    std::fs::write("debug_page.html", &html)?;
    println!("💾 HTML sauvegardé dans debug_page.html");

    // Liste des sélecteurs possibles à tester
    let possible_selectors = [
        // Sélecteurs courants pour les sites d'emploi
        ".job",
        ".job-card",
        ".job-listing",
        ".job-item",
        ".position",
        ".offer",
        ".card",
        ".listing",
        "article",
        "div[class*='job']",
        "div[class*='card']",
        "li[class*='job']",
        // Sélecteurs plus généraux
        "div > div > div", // Structure nested common
        "div.grid > div",  // Grid layouts
        "div.flex > div",  // Flex layouts
    ];

    println!("\n🔍 Test des sélecteurs CSS...");

    for selector_str in &possible_selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            let elements = document.select(&selector);
            let count = elements.count();
            
            if count > 0 {
                println!("✅ '{}': {} éléments trouvés", selector_str, count);
                
                // Réappliquer pour voir le contenu des premiers éléments
                let elements = document.select(&selector);
                for (i, element) in elements.enumerate().take(3) {
                    let text = element.text().collect::<String>();
                    let text_preview = if text.len() > 100 { 
                        &text[..100] 
                    } else { 
                        &text 
                    };
                    println!("   Élément {}: {}", i + 1, text_preview.replace("\n", " ").trim());
                }
            } else {
                println!("❌ '{}': 0 éléments", selector_str);
            }
        }
    }

    // Chercher des patterns spécifiques dans le HTML
    println!("\n🔎 Recherche de motifs spécifiques...");
    
    let patterns = ["rust", "developer", "engineer", "remote", "salary"];
    for pattern in patterns {
        if html.to_lowercase().contains(pattern) {
            let count = html.matches(pattern).count();
            println!("✅ Motif '{}' trouvé {} fois", pattern, count);
        }
    }

    // Afficher les balises avec classes
    println!("\n🏷️  Balises avec classes trouvées:");
    
    // Correction 1: Créer une liaison let pour le sélecteur
    let class_selector = Selector::parse("[class]").unwrap();
    let class_elements = document.select(&class_selector);
    
    for element in class_elements.take(20) {
        if let Some(class) = element.value().attr("class") {
            let tag_name = element.value().name();
            
            // Correction 2: Créer une liaison let pour le texte
            let text_content = element.text().collect::<String>();
            let text_preview = text_content.trim();
            
            let text_display = if text_preview.len() > 50 {
                format!("{}...", &text_preview[..50])
            } else {
                text_preview.to_string()
            };
            
            println!("   <{} class=\"{}\">: {}", tag_name, class, text_display);
        }
    }

    Ok(())
}