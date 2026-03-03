use comfy_table::presets::UTF8_FULL;
use comfy_table::{Attribute, Cell, Color, ContentArrangement, Table};
use console::style;
use crate::modrinth::ModrinthAPI;

pub async fn run(
    api: &ModrinthAPI, 
    query: String, 
    limit: i32, 
    page: i32, 
    sort: String, 
    facets: Option<String>
) -> Result<(), Box<dyn std::error::Error>> {
    
    let current_page = page.max(1);
    let offset = (current_page - 1) * limit;

    println!(
        "🔍 Searching for '{}' (Page: {}, Sort: {})...",
        style(&query).cyan(),
        style(current_page).yellow(),
        style(&sort).yellow()
    );

    let results = api.search(&query, limit, offset, &sort, facets).await?;

    if results.hits.is_empty() {
        println!("{}", style("No mods found.").red());
        return Ok(());
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["Project", "Slug", "Type", "Author", "Downloads"]);

    for hit in &results.hits {
        table.add_row(vec![
            Cell::new(&hit.title)
                .add_attribute(Attribute::Bold)
                .fg(Color::Cyan),
            Cell::new(&hit.slug).fg(Color::DarkGrey),
            Cell::new(&hit.project_type).fg(Color::Magenta),
            Cell::new(&hit.author),
            Cell::new(hit.downloads.to_string()).fg(Color::Green),
        ]);
    }

    println!("\n{table}");

    let total_pages = (results.total_hits as f32 / limit as f32).ceil() as i32;
    println!(
        "Page {} of {} | Total hits: {}",
        style(current_page).bold(),
        style(total_pages).bold(),
        style(results.total_hits).yellow()
    );

    Ok(())
}