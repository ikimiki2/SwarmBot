use std::fs::File;

use rand::seq::SliceRandom;
use serde::Deserialize;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};

use crate::bootstrap::csv::read_users;
use crate::bootstrap::mojang::{AuthResponse, Mojang};
use crate::bootstrap::opts::Opts;
use crate::error::{err, Error, HasContext, ResContext};
use std::time::Duration;
use itertools::Itertools;
use packets::types::UUID;
use crate::bootstrap::storage::{ProxyUser, ValidUser};
use tokio_socks::tcp::Socks5Stream;
use tokio::sync::mpsc::Receiver;

pub mod opts;
pub mod csv;
pub mod dns;
pub mod storage;
pub mod mojang;


#[derive(Clone, Debug)]
pub struct Address {
    pub host: String,
    pub port: u16,
}

impl From<&Address> for String {
    fn from(addr: &Address) -> Self {
        format!("{}:{}", addr.host, addr.port)
    }
}

#[derive(Debug)]
pub struct Connection {
    pub user: ValidUser,
    pub address: Address,
    pub mojang: Mojang,
    pub read: OwnedReadHalf,
    pub write: OwnedWriteHalf,
}

impl Connection {
    pub fn stream(address: Address, mut users: tokio::sync::mpsc::Receiver<ProxyUser>) -> Receiver<Connection> {

        let (tx,rx) = tokio::sync::mpsc::channel(1);
        tokio::spawn(async move {
           while let Some(user) = users.recv().await {
               let ProxyUser {proxy, user, mojang} = user;
               let target = String::from(&address);
               let conn = Socks5Stream::connect_with_password(proxy.address().as_str(), target.as_str(), &proxy.user, &proxy.pass).await.unwrap();
               let (read, write) = conn.into_inner().into_split();
               tx.send(Connection {
                   user, address: address.clone(), mojang, read, write
               }).await.unwrap();
           }
        });

        rx
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct CSVUser {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Proxy {
    pub host: String,
    pub port: u32,
    pub user: String,
    pub pass: String,
}

impl Proxy {
    pub fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

pub struct Output {
    pub version: usize,
    pub delay_millis: u64,
    pub connections: tokio::sync::mpsc::Receiver<Connection>,
}
