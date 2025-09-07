
use reqwest::Error;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, Serialize, Deserialize)]
struct JobPosting {
    title: String,
    company: String,
    location: String,
    tags: Vec<String>,
    url: String,
    description: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // URL du site à scraper
    let url = "https://rustjobs.dev/";
    
    println!("Récupération de la page principale...");
    
    // Faire la requête HTTP avec un user-agent
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .build()?;
    
    let resp = client.get(url).send().await?;
    let body = resp.text().await?;
    
    // Sauvegarder le HTML pour analyse
    let mut file = File::create("debug_page.html").unwrap();
    file.write_all(body.as_bytes()).unwrap();
    println!("Page HTML sauvegardée dans debug_page.html");
    
    // Parser le HTML
    let document = Html::parse_document(&body);
    
    // Essayer différents sélecteurs possibles basés sur l'analyse du HTML
    let selectors_to_try = [
        // Sélecteurs basés sur des patterns communs
        "div[class*='job']",
        "div[class*='card']",
        "a[href*='/jobs/']",
        "div[class*='list']",
        "article",
        "div.bg-white", // Les cartes ont souvent un fond blanc
    ];
    
    let mut job_postings = Vec::new();
    
    for selector_str in selectors_to_try.iter() {
        if let Ok(selector) = Selector::parse(selector_str) {
            let elements: Vec<_> = document.select(&selector).collect();
            println!("Sélecteur '{}' a trouvé {} éléments", selector_str, elements.len());
            
            if !elements.is_empty() {
                // Analyser le premier élément pour comprendre la structure
                if let Some(first_element) = elements.first() {
                    println!("Structure du premier élément:\n{:?}", first_element.html());
                }
            }
        }
    }
    
    // Sélecteurs plus spécifiques basés sur l'analyse visuelle
    // Ces sélecteurs sont des hypothèses et devront être ajustés
    let job_selector = Selector::parse("a[href^='/jobs/'], div.bg-white.rounded-lg.shadow, div.p-4.border.rounded").unwrap();
    
    println!("Recherche avec le sélecteur principal...");
    for job_element in document.select(&job_selector) {
        println!("Élément job trouvé: {:?}", job_element.value().attr("class"));
        
        // Extraire le titre - chercher les balises h2, h3, ou éléments avec classe de titre
        let title = extract_text(&job_element, "h2, h3, [class*='title'], [class*='name']");
        
        // Extraire le nom de l'entreprise
        let company = extract_text(&job_element, "[class*='company'], [class*='firm']");
        
        // Extraire le lieu
        let location = extract_text(&job_element, "[class*='location'], [class*='place']");
        
        // Extraire les tags
        let tags = extract_tags(&job_element, "[class*='tag'], .tag, .badge");
        
        // Extraire l'URL
        let url = job_element.value().attr("href")
            .map(|href| {
                if href.starts_with("http") {
                    href.to_string()
                } else {
                    format!("https://rustjobs.dev{}", href)
                }
            })
            .unwrap_or_else(|| {
                // Si l'élément n'est pas un lien, chercher un lien à l'intérieur
                if let Some(link) = job_element.select(&Selector::parse("a").unwrap()).next() {
                    link.value().attr("href").map(|href| {
                        if href.starts_with("http") {
                            href.to_string()
                        } else {
                            format!("https://rustjobs.dev{}", href)
                        }
                    }).unwrap_or_default()
                } else {
                    String::new()
                }
            });
        
        println!("Titre trouvé: {}", title);
        println!("Entreprise: {}", company);
        println!("URL: {}", url);
        println!("---");
        
        if !title.is_empty() && !url.is_empty() {
            // Pour une description plus détaillée, visiter chaque page d'offre
            let description = if !url.is_empty() && url != "https://rustjobs.dev" {
                sleep(Duration::from_millis(500)).await; // Respectueux du serveur
                fetch_job_description(&url).await.unwrap_or_default()
            } else {
                String::new()
            };
            
            // Créer et stocker le job posting
            job_postings.push(JobPosting {
                title,
                company,
                location,
                tags,
                url,
                description,
            });
        }
    }
    
    // Afficher les résultats
    println!("Nombre d'offres d'emploi trouvées: {}", job_postings.len());
    for job in &job_postings {
        println!("Titre: {}", job.title);
        println!("Entreprise: {}", job.company);
        println!("Lieu: {}", job.location);
        println!("Tags: {:?}", job.tags);
        println!("URL: {}", job.url);
        if !job.description.is_empty() {
            println!("Description: {}...", job.description.chars().take(100).collect::<String>());
        }
        println!("---");
    }
    
    // Sauvegarder dans un fichier JSON
    let json = serde_json::to_string_pretty(&job_postings).unwrap();
    let mut file = File::create("rustjobs.json").unwrap();
    file.write_all(json.as_bytes()).unwrap();
    println!("Données sauvegardées dans rustjobs.json");
    
    Ok(())
}

fn extract_text(element: &scraper::ElementRef, selector_str: &str) -> String {
    if let Ok(selector) = Selector::parse(selector_str) {
        element.select(&selector)
            .next()
            .map(|e| e.text().collect::<String>())
            .unwrap_or_default()
            .trim()
            .to_string()
    } else {
        String::new()
    }
}

fn extract_tags(element: &scraper::ElementRef, selector_str: &str) -> Vec<String> {
    let mut tags = Vec::new();
    
    if let Ok(selector) = Selector::parse(selector_str) {
        for tag_element in element.select(&selector) {
            let tag_text = tag_element.text().collect::<String>().trim().to_string();
            if !tag_text.is_empty() {
                tags.push(tag_text);
            }
        }
    }
    
    tags
}

async fn fetch_job_description(url: &str) -> Result<String, Error> {
    println!("Récupération de la description depuis: {}", url);
    
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .build()?;
    
    let resp = client.get(url).send().await?;
    let body = resp.text().await?;
    
    let document = Html::parse_document(&body);
    
    // Essayer différents sélecteurs pour la description
    let desc_selectors = [
        "div[class*='description']",
        "div[class*='content']",
        "article",
        "div.prose",
        "div.job-description",
        "main",
    ];
    
    for selector_str in desc_selectors.iter() {
        if let Ok(selector) = Selector::parse(selector_str) {
            if let Some(desc_element) = document.select(&selector).next() {
                let description = desc_element.text().collect::<String>();
                if !description.trim().is_empty() {
                    return Ok(description.trim().to_string());
                }
            }
        }
    }
    
    Ok(String::new())
}