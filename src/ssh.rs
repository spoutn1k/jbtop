use async_trait::async_trait;
use russh::*;
use russh_keys::*;
use std::error::Error;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::ToSocketAddrs;

static USER: &'static str = "jb";
static PORT: &'static str = "2222";
static SSH_KEY: &'static str = "/home/jb/.config/ssh/id_ed25519";

pub struct Client {}

#[async_trait]
impl client::Handler for Client {
    type Error = russh::Error;

    async fn check_server_key(
        self,
        _server_public_key: &key::PublicKey,
    ) -> Result<(Self, bool), Self::Error> {
        Ok((self, true))
    }
}

pub struct Session {
    pub handle: client::Handle<Client>,
}

pub struct Channel {
    pub channel: russh::Channel<client::Msg>,
}

impl Session {
    pub async fn new(hostname: &str) -> Result<Self, Box<dyn Error>> {
        Session::connect(SSH_KEY, USER, format!("{}:{}", hostname, PORT)).await
    }

    pub async fn connect<P: AsRef<Path>, A: ToSocketAddrs>(
        key_path: P,
        user: impl Into<String>,
        addrs: A,
    ) -> Result<Self, Box<dyn Error>> {
        let key_pair = load_secret_key(key_path, None)?;
        let config = client::Config {
            inactivity_timeout: Some(Duration::from_secs(5)),
            ..<_>::default()
        };

        let config = Arc::new(config);
        let sh = Client {};

        let mut handle = client::connect(config, addrs, sh).await?;
        let auth_res = handle
            .authenticate_publickey(user, Arc::new(key_pair))
            .await?;

        if !auth_res {
            log::error!("Failed to authenticate");
            return Err(String::from("Authentication error").into());
        }

        Ok(Self { handle })
    }

    pub async fn open_channel(&self) -> Result<Channel, Box<dyn Error>> {
        Ok(Channel {
            channel: self.handle.channel_open_session().await?,
        })
    }

    pub async fn close(&mut self) -> Result<(), Box<dyn Error>> {
        self.handle
            .disconnect(Disconnect::ByApplication, "", "English")
            .await?;
        Ok(())
    }
}

impl Channel {
    pub async fn block_exec(
        &mut self,
        command: &str,
    ) -> Result<(u32, String, String), Box<dyn Error>> {
        self.channel.exec(true, command).await?;

        let mut code = None;
        let mut stdout = String::new();
        let mut stderr = String::new();

        loop {
            let Some(msg) = self.channel.wait().await else {
                break;
            };

            match msg {
                ChannelMsg::Data { ref data } => {
                    stdout.push_str(
                        &String::from_utf8(data.iter().cloned().collect())
                            .expect("Invalid output !"),
                    );
                }

                ChannelMsg::ExtendedData { ref data, ext } if ext == 1 => {
                    let error = String::from_utf8(data.iter().cloned().collect())
                        .expect("Invalid output !");
                    let stripped = error.strip_suffix("\n").unwrap_or(&error);
                    stderr.push_str(stripped);
                }

                ChannelMsg::ExitStatus { exit_status } => {
                    code = Some(exit_status);
                }

                _ => (),
            }
        }

        match code {
            Some(value) => Ok((value, stdout, stderr)),
            None => Err("Program did not exit cleanly !".into()),
        }
    }
}

/*
pub async fn check(&self, command: &str) -> Result<(u32, String, String), Box<dyn Error>> {
    let mut channel = self.handle.channel_open_session().await?;
    channel.exec(true, command).await?;

    let mut code = None;
    let mut stdout = String::new();
    let mut stderr = String::new();

    loop {
        let Some(msg) = channel.wait().await else {
            break;
        };

        match msg {
            ChannelMsg::Data { ref data } => {
                stdout.push_str(
                    &String::from_utf8(data.iter().cloned().collect()).expect("Invalid output !"),
                );
            }

            ChannelMsg::ExtendedData { ref data, ext } if ext == 1 => {
                let error =
                    String::from_utf8(data.iter().cloned().collect()).expect("Invalid output !");
                let stripped = error.strip_suffix("\n").unwrap_or(&error);
                stderr.push_str(stripped);
            }

            ChannelMsg::ExitStatus { exit_status } => {
                code = Some(exit_status);
            }

            _ => (),
        }
    }

    Ok((code.expect("Program did not exit cleanly"), stdout, stderr))
}

pub async fn stream<S: From<(ChannelId, ChannelMsg)> + Sync + std::marker::Send + 'static>(
    channel: &mut Channel<S>,
    command: &str,
) -> Result<u32, Box<dyn Error>> {
    channel.exec(true, command).await?;

    let mut code = None;
    let mut stdout = tokio::io::stdout();
    let mut stderr = tokio::io::stderr();

    let mut buffer: Vec<u8> = vec![];

    loop {
        let Some(msg) = channel.wait().await else {
            break;
        };

        match msg {
            ChannelMsg::Data { ref data } => {
                for c in data.iter() {
                    match c {
                        b'\n' => {
                            buffer.push(b'\n');
                            stdout.write(&buffer).await?;
                            stdout.flush().await?;
                            buffer.clear();
                        }
                        _ => buffer.push(*c),
                    }
                }
            }

            ChannelMsg::ExtendedData { ref data, ext } if ext == 1 => {
                stderr.write_all(data).await?;
                stderr.flush().await?;
            }

            ChannelMsg::ExitStatus { exit_status } => {
                code = Some(exit_status);
            }

            _ => (),
        }
    }

    Ok(code.expect("Program did not exit cleanly"))
}
*/
