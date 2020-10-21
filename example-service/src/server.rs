// Copyright 2018 Google LLC
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

use clap::{App, Arg};
use futures::{future, prelude::*};
use service::World;
use std::{
    io,
    net::{IpAddr, SocketAddr},
};
use tarpc::{
    context,
    server::{self, Channel, Handler},
};
use tokio::net::TcpListener;
use tokio_util::codec::Framed;
use tokio_util::codec::length_delimited::LengthDelimitedCodec;
use tokio_serde::formats::Json;

// This is the type that implements the generated World trait. It is the business logic
// and is used to start the server.
#[derive(Clone)]
struct HelloServer(SocketAddr);

#[tarpc::server]
impl World for HelloServer {
    async fn hello(self, context: context::Context, name: String) -> String {
        println!("hello({:?}, {}", context, name);
        format!("Hello, {}! You are connected from {:?}.", name, self.0)
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    env_logger::init();

    let flags = App::new("Hello Server")
        .version("0.1")
        .author("Tim <tikue@google.com>")
        .about("Say hello!")
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .value_name("NUMBER")
                .help("Sets the port number to listen on")
                .required(true)
                .takes_value(true),
        )
        .get_matches();

    let port = flags.value_of("port").unwrap();
    let port = port
        .parse()
        .unwrap_or_else(|e| panic!(r#"--port value "{}" invalid: {}"#, port, e));

    let server_addr = SocketAddr::new(IpAddr::from([0, 0, 0, 0]), port);

    let listener = TcpListener::bind(server_addr).await?;
    let (stream, client_addr) = listener.accept().await?;
    println!("Accepted connection from {}", client_addr);
    let framed_stream = Framed::new(stream, LengthDelimitedCodec::default());
    let server_transport = tarpc::serde_transport::new(framed_stream, Json::default());

    let server = server::new(server::Config::default())
        .incoming(stream::once(future::ready(server_transport)))
        .respond_with(HelloServer(client_addr).serve());

    server.await;
    println!("server.await returned. It should only return once `stream` has been fully processed?");

    // This hack makes the server respond correctly before terminating
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    Ok(())
}
