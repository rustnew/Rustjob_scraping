use reqwest::Client;
use scraper::{Html, Selector, ElementRef};
use serde::Serialize;
use std::error::Error;
use std::fs::File;
use csv::Writer;
use std::time::Duration;
use regex::Regex;
use std::collections::HashSet;

#[derive(Debug, Serialize, Default, Clone)]
struct JobPosting {
    title: String,
    company: String,
    location: String,
    salary: String,
    job_type: String,
    experience_level: String,
    remote: String,
    technologies: String,
    description: String,
    url: String,
    date_posted: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🚀 Démarrage du scraping optimisé de RustJobs.dev...");
    
    let client = Client::builder()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .timeout(Duration::from_secs(30))
        .build()?;

    let mut all_jobs = Vec::new();
    
    // Scraper plusieurs pages
    let base_urls = vec![
        "https://rustjobs.dev/",
        "https://rustjobs.dev/jobs",
    ];
    
    for url in base_urls {
        println!("📡 Scraping: {}", url);
        
        match scrape_page(&client, url).await {
            Ok(mut jobs) => {
                println!("✅ Trouvé {} offres sur {}", jobs.len(), url);
                all_jobs.append(&mut jobs);
            }
            Err(e) => {
                println!("❌ Erreur sur {}: {}", url, e);
            }
        }
        
        tokio::time::sleep(Duration::from_millis(1000)).await;
    }
    
    // Dédoublonner
    all_jobs = deduplicate_jobs(all_jobs);
    
    if all_jobs.is_empty() {
        println!("⚠️  Aucune offre trouvée. Le site utilise probablement du JavaScript dynamique.");
        analyze_dynamic_content();
    } else {
        save_to_csv(&all_jobs, "rustjobs_optimized.csv")?;
        display_results(&all_jobs);
    }
    
    Ok(())
}

async fn scrape_page(client: &Client, url: &str) -> Result<Vec<JobPosting>, Box<dyn Error>> {
    let response = client.get(url).send().await?;
    let html = response.text().await?;
    let document = Html::parse_document(&html);
    
    let mut jobs = Vec::new();
    
    // Multiples stratégies d'extraction
    extract_from_job_containers(&document, &mut jobs);
    extract_from_cards(&document, &mut jobs);
    extract_from_lists(&document, &mut jobs);
    extract_from_text_patterns(&document, &mut jobs, url);
    
    Ok(jobs)
}

fn extract_from_job_containers(document: &Html, jobs: &mut Vec<JobPosting>) {
    let job_selectors = [
        ".job-listing", ".job-card", ".job-item", "[data-job-id]",
        ".listing", "article", ".card", ".post", "[class*='job']", "[id*='job']"
    ];
    
    for selector_str in job_selectors.iter() {
        if let Ok(selector) = Selector::parse(selector_str) {
            for element in document.select(&selector) {
                if let Some(job) = extract_job_from_container(element) {
                    jobs.push(job);
                }
            }
        }
    }
}

fn extract_from_cards(document: &Html, jobs: &mut Vec<JobPosting>) {
    let card_patterns = [
        "div[class*='bg-white'][class*='shadow']",
        "div[class*='border'][class*='rounded']",
        "div[class*='p-'][class*='space-y']",
        "div[class*='grid'][class*='gap']"
    ];
    
    for pattern in card_patterns.iter() {
        if let Ok(selector) = Selector::parse(pattern) {
            for element in document.select(&selector) {
                let text: String = element.text().collect();
                
                if contains_job_indicators(&text) && text.len() > 50 {
                    if let Some(job) = extract_job_smart(element, &text) {
                        jobs.push(job);
                    }
                }
            }
        }
    }
}

fn extract_from_lists(document: &Html, jobs: &mut Vec<JobPosting>) {
    if let Ok(li_selector) = Selector::parse("li") {
        for element in document.select(&li_selector) {
            let text: String = element.text().collect();
            
            if contains_job_indicators(&text) && text.len() > 30 {
                if let Some(job) = extract_job_smart(element, &text) {
                    jobs.push(job);
                }
            }
        }
    }
}

fn extract_from_text_patterns(document: &Html, jobs: &mut Vec<JobPosting>, base_url: &str) {
    let job_pattern = Regex::new(r"(?i)(rust\s+(?:developer|engineer|programmer))").unwrap();
    
    if let Ok(selector) = Selector::parse("div, section, article") {
        for element in document.select(&selector) {
            let text: String = element.text().collect();
            
            if job_pattern.is_match(&text) && text.len() > 100 {
                let title = extract_title_with_regex(&text);
                let company = extract_company_with_regex(&text);
                let location = extract_location_with_regex(&text);
                
                if !title.is_empty() {
                    let job = JobPosting {
                        title,
                        company,
                        location,
                        technologies: extract_technologies(&text),
                        description: truncate_description(&text),
                        url: extract_job_url(element, base_url),
                        remote: if text.to_lowercase().contains("remote") { "Yes".to_string() } else { "No".to_string() },
                        ..Default::default()
                    };
                    
                    jobs.push(job);
                }
            }
        }
    }
}

fn extract_job_from_container(element: ElementRef) -> Option<JobPosting> {
    let text: String = element.text().collect();
    
    if !contains_job_indicators(&text) || is_navigation_element(&text) {
        return None;
    }
    
    extract_job_smart(element, &text)
}

fn extract_job_smart(element: ElementRef, text: &str) -> Option<JobPosting> {
    let title = find_job_title(element, text);
    
    if title.is_empty() || title.len() < 5 {
        return None;
    }
    
    Some(JobPosting {
        title,
        company: find_company_name(element, text),
        location: find_location(element, text),
        technologies: extract_technologies(text),
        description: truncate_description(text),
        url: extract_job_url(element, "https://rustjobs.dev"),
        salary: extract_salary(text),
        job_type: extract_job_type(text),
        experience_level: extract_experience_level(text),
        remote: if text.to_lowercase().contains("remote") { "Yes".to_string() } else { "No".to_string() },
        date_posted: extract_date(text),
    })
}

fn find_job_title(element: ElementRef, text: &str) -> String {
    let header_selectors = ["h1", "h2", "h3", "h4", "h5", "h6"];
    for sel_str in header_selectors.iter() {
        if let Ok(selector) = Selector::parse(sel_str) {
            for header in element.select(&selector) {
                let header_text: String = header.text().collect();
                if is_job_title(&header_text) {
                    return clean_text(&header_text);
                }
            }
        }
    }
    
    extract_title_with_regex(text)
}

fn find_company_name(element: ElementRef, text: &str) -> String {
    let company_selectors = ["[class*='company']", "[class*='employer']", ".font-semibold"];
    
    for sel_str in company_selectors.iter() {
        if let Ok(selector) = Selector::parse(sel_str) {
            for comp_element in element.select(&selector) {
                let comp_text: String = comp_element.text().collect();
                if is_company_name(&comp_text) {
                    return clean_text(&comp_text);
                }
            }
        }
    }
    
    extract_company_with_regex(text)
}

fn find_location(element: ElementRef, text: &str) -> String {
    let location_selectors = ["[class*='location']", "[class*='place']", "[class*='remote']"];
    
    for sel_str in location_selectors.iter() {
        if let Ok(selector) = Selector::parse(sel_str) {
            for loc_element in element.select(&selector) {
                let loc_text: String = loc_element.text().collect();
                if is_location(&loc_text) {
                    return clean_text(&loc_text);
                }
            }
        }
    }
    
    extract_location_with_regex(text)
}

fn contains_job_indicators(text: &str) -> bool {
    let indicators = ["rust developer", "rust engineer", "backend", "software engineer", "remote"];
    let text_lower = text.to_lowercase();
    indicators.iter().any(|&indicator| text_lower.contains(indicator))
}

fn is_navigation_element(text: &str) -> bool {
    let nav_indicators = ["post job", "start hiring", "about us", "blog", "sign in", "sign up"];
    let text_lower = text.to_lowercase();
    nav_indicators.iter().any(|&indicator| text_lower.contains(indicator))
}

fn is_job_title(text: &str) -> bool {
    let text = text.trim();
    text.len() > 10 && text.len() < 150 && text.to_lowercase().contains("rust")
}

fn is_company_name(text: &str) -> bool {
    let text = text.trim();
    if text.len() < 2 || text.len() > 50 { return false; }
    
    let excluded_words = ["rust", "developer", "engineer", "software", "remote"];
    let text_lower = text.to_lowercase();
    if excluded_words.iter().any(|&word| text_lower.contains(word)) { return false; }
    
    let first_char = text.chars().next().unwrap_or(' ');
    first_char.is_uppercase()
}

fn is_location(text: &str) -> bool {
    let text = text.trim();
    if text.len() < 2 || text.len() > 30 { return false; }
    
    let location_indicators = ["remote", "hybrid", "on-site", "europe", "usa", "uk"];
    let text_lower = text.to_lowercase();
    location_indicators.iter().any(|&indicator| text_lower.contains(indicator))
}

fn extract_title_with_regex(text: &str) -> String {
    let patterns = [
        r"(?i)(senior\s+rust\s+(?:developer|engineer))",
        r"(?i)(rust\s+(?:developer|engineer|programmer))",
        r"(?i)(backend\s+(?:developer|engineer).*rust)",
    ];
    
    for pattern in patterns.iter() {
        if let Ok(re) = Regex::new(pattern) {
            if let Some(cap) = re.captures(text) {
                if let Some(title) = cap.get(1) {
                    return clean_text(title.as_str());
                }
            }
        }
    }
    
    for line in text.lines() {
        let line = line.trim();
        if line.len() > 10 && line.to_lowercase().contains("rust") {
            return clean_text(line);
        }
    }
    
    String::new()
}

fn extract_company_with_regex(text: &str) -> String {
    let company_pattern = r"\b([A-Z][a-zA-Z&\s,\.]{1,25}(?:\s+(?:Inc|LLC|Ltd|GmbH)?\.?))\b";
    
    if let Ok(re) = Regex::new(company_pattern) {
        for cap in re.captures_iter(text) {
            if let Some(company) = cap.get(1) {
                let company_text = company.as_str().trim();
                if is_company_name(company_text) {
                    return clean_text(company_text);
                }
            }
        }
    }
    
    String::new()
}

fn extract_location_with_regex(text: &str) -> String {
    let location_patterns = [
        r"(?i)\b(remote)\b",
        r"(?i)\b(hybrid)\b", 
        r"(?i)\b(on-?site)\b",
        r"(?i)\b(berlin|london|paris|amsterdam)\b",
    ];
    
    for pattern in location_patterns.iter() {
        if let Ok(re) = Regex::new(pattern) {
            if let Some(cap) = re.captures(text) {
                if let Some(location) = cap.get(1) {
                    return clean_text(location.as_str());
                }
            }
        }
    }
    
    String::new()
}

fn extract_technologies(text: &str) -> String {
    let tech_keywords = ["Rust", "WebAssembly", "Tokio", "PostgreSQL", "Docker", "AWS", "React"];
    let mut found_techs = HashSet::new();
    let text_lower = text.to_lowercase();
    
    for tech in tech_keywords.iter() {
        if text_lower.contains(&tech.to_lowercase()) {
            found_techs.insert(tech.to_string());
        }
    }
    
    found_techs.into_iter().collect::<Vec<_>>().join(", ")
}

fn extract_salary(text: &str) -> String {
    let salary_patterns = [r"(?i)\$(\d{2,3}[,.]?\d{3})\s*(?:-\s*\$?(\d{2,3}[,.]?\d{3}))?"];
    
    for pattern in salary_patterns.iter() {
        if let Ok(re) = Regex::new(pattern) {
            if let Some(cap) = re.captures(text) {
                return cap.get(0).unwrap().as_str().to_string();
            }
        }
    }
    
    String::new()
}

fn extract_job_type(text: &str) -> String {
    let text_lower = text.to_lowercase();
    
    if text_lower.contains("full-time") { "Full-time".to_string() }
    else if text_lower.contains("part-time") { "Part-time".to_string() }
    else if text_lower.contains("contract") { "Contract".to_string() }
    else { String::new() }
}

fn extract_experience_level(text: &str) -> String {
    let text_lower = text.to_lowercase();
    
    if text_lower.contains("senior") { "Senior".to_string() }
    else if text_lower.contains("junior") { "Junior".to_string() }
    else if text_lower.contains("lead") { "Lead".to_string() }
    else { String::new() }
}

fn extract_date(text: &str) -> String {
    let date_patterns = [r"(?i)(\d{1,2}/\d{1,2}/\d{4})", r"(?i)(posted\s+\d+\s+days?\s+ago)"];
    
    for pattern in date_patterns.iter() {
        if let Ok(re) = Regex::new(pattern) {
            if let Some(cap) = re.captures(text) {
                return cap.get(1).unwrap().as_str().to_string();
            }
        }
    }
    
    String::new()
}

fn extract_job_url(element: ElementRef, base_url: &str) -> String {
    if let Ok(link_selector) = Selector::parse("a[href]") {
        if let Some(link) = element.select(&link_selector).next() {
            if let Some(href) = link.value().attr("href") {
                if href.starts_with("http") {
                    return href.to_string();
                } else if href.starts_with("/") {
                    return format!("{}{}", base_url.trim_end_matches('/'), href);
                }
            }
        }
    }
    
    String::new()
}

fn clean_text(text: &str) -> String {
    text.trim()
        .replace('\n', " ")
        .replace('\t', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn truncate_description(text: &str) -> String {
    let cleaned = clean_text(text);
    if cleaned.len() > 300 {
        format!("{}...", &cleaned[..297])
    } else {
        cleaned
    }
}

fn deduplicate_jobs(jobs: Vec<JobPosting>) -> Vec<JobPosting> {
    let mut unique_jobs = Vec::new();
    let mut seen_titles = HashSet::new();
    
    for job in jobs {
        let key = format!("{}-{}", job.title.to_lowercase(), job.company.to_lowercase());
        if !seen_titles.contains(&key) {
            seen_titles.insert(key);
            unique_jobs.push(job);
        }
    }
    
    unique_jobs
}

fn analyze_dynamic_content() {
    println!("\n💡 Le site utilise probablement du JavaScript dynamique (Gatsby.js)");
    println!("📋 Suggestions:");
    println!("   - Utiliser un navigateur headless (Playwright/Puppeteer)");
    println!("   - Inspecter les requêtes réseau pour trouver l'API");
    println!("   - Vérifier si le site expose une API GraphQL");
}

fn save_to_csv(jobs: &[JobPosting], filename: &str) -> Result<(), Box<dyn Error>> {
    let file = File::create(filename)?;
    let mut writer = Writer::from_writer(file);

    writer.write_record(&[
        "Titre", "Entreprise", "Localisation", "Salaire", 
        "Type", "Niveau", "Remote", "Technologies", "Description", "URL", "Date"
    ])?;

    for job in jobs {
        writer.write_record(&[
            &job.title,
            &job.company,
            &job.location,
            &job.salary,
            &job.job_type,
            &job.experience_level,
            &job.remote,
            &job.technologies,
            &job.description,
            &job.url,
            &job.date_posted
        ])?;
    }

    writer.flush()?;
    Ok(())
}

fn display_results(jobs: &[JobPosting]) {
    println!("\n📊 RÉSULTATS DU SCRAPING:");
    println!("============================================================");
    println!("📋 Total des offres trouvées: {}", jobs.len());
    
    if !jobs.is_empty() {
        let remote_count = jobs.iter().filter(|j| j.remote == "Yes").count();
        let with_salary = jobs.iter().filter(|j| !j.salary.is_empty()).count();
        
        println!("🏠 Offres remote: {}", remote_count);
        println!("💰 Avec salaire: {}", with_salary);
        
        println!("\n📋 OFFRES DÉTAILLÉES:");
        println!("============================================================");
        
        for (i, job) in jobs.iter().enumerate() {
            println!("\n🔹 OFFRE #{}", i + 1);
            println!("📝 Titre: {}", job.title);
            println!("🏢 Entreprise: {}", if job.company.is_empty() { "Non spécifiée" } else { &job.company });
            println!("📍 Localisation: {}", if job.location.is_empty() { "Non spécifiée" } else { &job.location });
            println!("💰 Salaire: {}", if job.salary.is_empty() { "Non spécifié" } else { &job.salary });
            println!("🏠 Remote: {}", job.remote);
            println!("🛠️  Technologies: {}", if job.technologies.is_empty() { "Non spécifiées" } else { &job.technologies });
            println!("🔗 URL: {}", if job.url.is_empty() { "Non disponible" } else { &job.url });
            println!("📄 Description: {}", 
                if job.description.len() > 100 { 
                    format!("{}...", &job.description[..97]) 
                } else { 
                    job.description.clone() 
                }
            );
            println!("----------------------------------------------------");
        }
        
        // Technologies les plus populaires
        let mut tech_count = std::collections::HashMap::new();
        for job in jobs {
            for tech in job.technologies.split(", ") {
                if !tech.trim().is_empty() {
                    *tech_count.entry(tech.trim().to_string()).or_insert(0) += 1;
                }
            }
        }
        
        if !tech_count.is_empty() {
            println!("\n🔧 TECHNOLOGIES LES PLUS DEMANDÉES:");
            let mut sorted_techs: Vec<_> = tech_count.iter().collect();
            sorted_techs.sort_by(|a, b| b.1.cmp(a.1));
            
            for (tech, count) in sorted_techs.iter().take(10) {
                println!("   {} : {} offres", tech, count);
            }
        }
    }
    
    println!("\n✅ Scraping terminé!");
}