use std::fs::File;

use itertools::Itertools;
use tokio::net::TcpStream;
use tokio_socks::tcp::Socks5Stream;

use crate::bootstrap::{Connection, User};
use crate::bootstrap::csv::read_proxies;
use crate::bootstrap::mojang::Mojang;
use crate::error::{HasContext, ResContext};

pub async fn obtain_connections(proxy: bool, proxies: &str, host: &str, port: u16, users: &[User]) -> ResContext<tokio::sync::mpsc::Receiver<Connection>> {
    let host = String::from(host);
    // TODO: is there a drain method instead
    let users = users.to_vec();

    let addr = format!("{}:{}", host, port);

    let count = users.len();
    println!("count is {}", count);

    let (tx, rx) = tokio::sync::mpsc::channel(32);

    match proxy {
        false => {
            tokio::task::spawn_local(async move {
                for user in users {
                    let stream = TcpStream::connect(&addr).await.context(|| format!("connecting to server")).unwrap();
                    let mojang = Mojang::default();
                    tx.send(combine(user, stream, mojang, host.clone(), port)).await.unwrap();
                }
            });
        }
        true => {
            let file = File::open(proxies).context(|| format!("opening proxy ({})", proxies))?;
            let proxies = read_proxies(file).context(|| format!("opening proxies ({})", proxies))?;

            tokio::task::spawn_local(async move {
                for (proxies, user) in proxies.chunks_exact(2).cycle().zip(users) {
                    let stream = {
                        let proxy = &proxies[0];
                        let proxy_addr = proxy.address();
                        let actual_addr = format!("{}:{}", host, port);
                        let stream = Socks5Stream::connect_with_password(proxy_addr.as_str(), actual_addr.as_str(), &proxy.user, &proxy.pass).await.unwrap();
                        stream.into_inner()
                        // stream.await.context(|| format!("connecting to proxy {}", proxy.address()))?
                    };

                    let mojang = {
                        let proxy = &proxies[1];
                        let proxy_addr = proxy.address();
                        Mojang::socks5(proxy_addr.as_str(), &proxy.user, &proxy.pass).context(|| format!("generating mojang https client")).unwrap()
                    };

                    tx.send(combine(user, stream, mojang, host.clone(), port)).await.unwrap();
                }
            });
        }
    }

    Ok(rx)
}

fn combine(user: User, stream: TcpStream, mojang: Mojang, host: String, port: u16) -> Connection {
    let online = user.online;
    let (read, write) = stream.into_split();
    Connection {
        mojang,
        user,
        online,
        host,
        read,
        write,
        port,
    }
}
