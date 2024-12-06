use rss::Item;

#[derive(Clone, Debug)]
pub struct UpdateMessage {
    pub item: Item,
    pub targets: Vec<usize>,
}
