//! Client for tldr-pages writting in Rust

#![deny(missing_docs,
        missing_debug_implementations, missing_copy_implementations,
        trivial_casts, trivial_numeric_casts,
        unsafe_code,
        unstable_features,
        unused_import_braces, unused_qualifications)]
#![cfg_attr(feature = "dev", allow(unstable_features))]
#![cfg_attr(feature = "dev", feature(plugin))]
#![cfg_attr(feature = "dev", plugin(clippy))]

extern crate hyper;
extern crate clap;
#[macro_use]
extern crate quick_error;

use std::io::{stderr, Read, Write};
use clap::{Arg, App};
use hyper::client::response::Response as HttpResponse;

#[cfg(target_os = "macos")]
const PLATFORM: &'static str = "osx";
#[cfg(not(target_os = "macos"))]
const PLATFORM: &'static str = "linux";

fn fetch_tldr(command: &str, platform: &str) -> Result<HttpResponse, Error> {
    let client = hyper::Client::new();
    let url = format!("http://raw.github.com/rprieto/tldr/master/pages/{platform}/{page}.md",
                      platform = platform,
                      page = command);

    let res = try!(client.get(&url)
                         .header(hyper::header::Connection::close())
                         .header(hyper::header::ContentType::plaintext())
                         .send()
                         .map_err(|err| Error::HttpRequest(command.to_owned(), err)));

    if res.status.is_success() {
        Ok(res)
    } else {
        Err(Error::HttpReponse(command.to_owned(), res.status))
    }
}

fn render_tldr<R: Read>(text: &mut R) -> Result<(), Error> {
    let mut buf = String::with_capacity(1024);
    try!(text.read_to_string(&mut buf));

    println!("{}", buf);
    Ok(())
}

quick_error! {
    #[derive(Debug)]
    enum Error {
        HttpRequest(command: String, err: hyper::Error) {
            from(err)
            cause(err)
            description("Error fetching page")
            display("Error fetching description for `{}`: {}", command, err)
        }
        HttpReponse(command: String, status: hyper::status::StatusCode) {
            description("Could not fetch description for command")
            display("Could not fetch description for command `{}`: {}", command, status)
        }
        Io(err: std::io::Error) {
            from()
            cause(err)
            description(err.description())
            display("Couldn't read stream")
        }
    }
}

fn main() {
    let matches = App::new("tldr")
                      .version(env!("CARGO_PKG_VERSION"))
                      .author("Pascal Hertleif <killercup@gmail.com>")
                      .about("Simplified and community-driven man pages")
                      .arg(Arg::with_name("command")
                               .help("Fetch the docs for command and render them to the \
                                      terminal.")
                               .required(true)
                               .index(1))
                      .get_matches();

    let command = matches.value_of("command").unwrap();

    fetch_tldr(command, "common")
        .or_else(|_| fetch_tldr(command, PLATFORM))
        .and_then(|mut res| render_tldr(&mut res))
        .unwrap_or_else(|err| {
            writeln!(stderr(), "{}", err).unwrap();
            std::process::exit(1);
        });
}
