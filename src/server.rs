use axum::extract::FromRef;
use dashmap::DashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::{broadcast, oneshot};

use crate::{
    config::Config,
    message::UpdateMessage,
    subscription::{load_subscriptions, Subscription},
    target::{load_targets, Target},
};

type SharedLock<T> = Arc<RwLock<T>>;
type SharedMap<K, V> = Arc<DashMap<K, V>>;

#[derive(Debug)]
pub enum HandlerType {
    Subscription,
    Target,
}

#[derive(Debug, Clone, FromRef)]
pub struct AppState {
    pub config: SharedLock<Config>,
    pub broadcast_sender: broadcast::Sender<UpdateMessage>,
    pub targets: SharedLock<Vec<Target>>,
    pub subscriptions: SharedLock<Vec<Subscription>>,
    pub handlers: SharedMap<
        usize,
        (
            HandlerType,
            tokio::task::JoinHandle<()>,
            oneshot::Sender<()>,
        ),
    >,
}

// TODO: statistics
// TODO: log retrieval
// TODO: auth
// TODO: dark mode

pub fn server_init(config: Config) -> AppState {
    // make AppState
    let (broadcast_sender, _) = broadcast::channel(config.server.queue_size);
    let targets = load_targets();
    let mut subscriptions = load_subscriptions();
    let handlers = DashMap::new();

    // spawn target handlers and subscription handlers
    for target in targets.iter() {
        let (kill_sender, kill_receiver) = oneshot::channel();
        let recv = broadcast_sender.subscribe();
        let handler = tokio::spawn(handle_target(target.clone(), recv, kill_receiver));
        handlers.insert(target.id, (HandlerType::Target, handler, kill_sender));
    }
    // spawn subscription handlers
    for subscription in subscriptions.iter_mut() {
        let (kill_sender, kill_receiver) = oneshot::channel();
        let broadcast_sender = broadcast_sender.clone();
        let handler = tokio::spawn(handle_subscription(
            subscription.clone(),
            broadcast_sender,
            kill_receiver,
        ));
        handlers.insert(
            subscription.id,
            (HandlerType::Subscription, handler, kill_sender),
        );
    }

    AppState {
        config: Arc::new(RwLock::new(config)),
        broadcast_sender,
        targets: Arc::new(RwLock::new(targets)),
        subscriptions: Arc::new(RwLock::new(subscriptions)),
        handlers: Arc::new(handlers),
    }
}

async fn handle_target(
    // move occurs here because we have to kill the handler when target updates anyway
    target: Target,
    mut recv: broadcast::Receiver<UpdateMessage>,
    mut halt: oneshot::Receiver<()>,
) {
    // TODO: push interval handling
    // TODO: templating
    // TODO: actually pushing
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
    mut subscription: Subscription,
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
    check_update(&mut subscription, &sender).await;
    loop {
        tokio::select! {
            _ = &mut halt => break,
            _ = tokio::time::sleep(tokio::time::Duration::from_secs(subscription.interval)) => {
                check_update(&mut subscription, &sender).await;
            }
        }
    }
}

impl AppState {
    pub fn update_config(&self, config: Config) {
        let mut guard = self.config.write().unwrap();
        *guard = config;
        // TODO: restart server(is it practical?)
    }

    pub fn add_target(&self, target: Target) {
        let (kill_sender, kill_receiver) = oneshot::channel();
        let recv = self.broadcast_sender.subscribe();
        let handler = tokio::spawn(handle_target(target.clone(), recv, kill_receiver));
        self.handlers
            .insert(target.id, (HandlerType::Target, handler, kill_sender));
        let mut guard = self.targets.write().unwrap();
        guard.push(target);
    }

    pub fn remove_target(&self, id: usize) -> anyhow::Result<()> {
        if let Some((_, (HandlerType::Target, _, kill_sender))) = self.handlers.remove(&id) {
            kill_sender.send(()).unwrap();
        } else {
            return Err(anyhow::anyhow!("Target not found."));
        }
        let mut guard = self.targets.write().unwrap();
        guard.retain(|target| target.id != id);
        Ok(())
    }

    pub fn add_subscription(&self, subscription: Subscription) {
        let (kill_sender, kill_receiver) = oneshot::channel();
        let broadcast_sender = self.broadcast_sender.clone();
        let handler = tokio::spawn(handle_subscription(
            subscription.clone(),
            broadcast_sender,
            kill_receiver,
        ));
        self.handlers.insert(
            subscription.id,
            (HandlerType::Subscription, handler, kill_sender),
        );
        let mut guard = self.subscriptions.write().unwrap();
        guard.push(subscription);
    }

    pub fn remove_subscription(&self, id: usize) -> anyhow::Result<()> {
        if let Some((_, (HandlerType::Subscription, _, kill_sender))) = self.handlers.remove(&id) {
            kill_sender.send(()).unwrap();
        } else {
            return Err(anyhow::anyhow!("Subscription not found."));
        }
        let mut guard = self.subscriptions.write().unwrap();
        guard.retain(|subscription| subscription.id != id);
        Ok(())
    }
}
