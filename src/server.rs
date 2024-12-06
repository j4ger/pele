use std::sync::LazyLock;

use tokio::sync::{broadcast, oneshot};

use crate::{
    config::Config,
    message::UpdateMessage,
    subscription::{load_subscriptions, Subscription},
    target::{load_targets, Target},
};

pub static CONFIG: LazyLock<Config> = LazyLock::new(|| Config::new().unwrap());

pub static SUBSCRIPTIONS: LazyLock<Vec<Subscription>> =
    LazyLock::new(|| load_subscriptions().unwrap());

pub static TARGETS: LazyLock<Vec<Target>> = LazyLock::new(|| load_targets().unwrap());

pub async fn server_init() {}

async fn handle_target(
    target: Target,
    mut recv: broadcast::Receiver<UpdateMessage>,
    mut halt: oneshot::Receiver<()>,
) {
    loop {
        tokio::select! {
            _ = &mut halt => break,
            message = recv.recv() => {
                if let Ok(message) = message {
                    if !message.targets.contains(&target.id) {
                        continue;
                    }
                    println!("{} updated.", message.item.title().unwrap_or("default_title"));
                }
            }
        }
    }
}

async fn handle_subscription(
    subscription: &mut Subscription,
    sender: broadcast::Sender<UpdateMessage>,
    mut halt: oneshot::Receiver<()>,
) {
    async fn check_update(
        subscription: &mut Subscription,
        sender: &broadcast::Sender<UpdateMessage>,
    ) {
        if let Some(items) = subscription.fetch().await.unwrap() {
            for item in items {
                let message = UpdateMessage {
                    item,
                    targets: subscription.push_targets.clone(),
                };
                // TODO: error handling
                sender.send(message).unwrap();
            }
        }
    }
    check_update(subscription, &sender).await;
    loop {
        tokio::select! {
            _ = &mut halt => break,
            _ = tokio::time::sleep(tokio::time::Duration::from_secs(subscription.interval)) => {
                check_update(subscription, &sender).await;
            }
        }
    }
}
