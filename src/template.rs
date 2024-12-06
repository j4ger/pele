use rss::Item;

pub fn render(template: &str, item: Item) -> String {
    format!(
        "{} updated.",
        item.title().unwrap_or("default_title".into())
    )
}
