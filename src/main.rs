use clap::{App, Arg};
use crates_io::{Crate, Registry};
use curl::easy::Easy;
use eyre::{Result, WrapErr};
use std::borrow::Cow;
use std::io;

const HOST: &str = "https://crates.io";
const DOCS: &str = "https://docs.rs";

fn main() -> Result<()> {
    // access metadata from cargo package http://stackoverflow.com/a/27841363/745121
    let args = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(Arg::with_name("query").short("q").required(true).index(1))
        .get_matches();

    let mut easy_client = Easy::new();
    let useragent = [
        env!("CARGO_PKG_NAME"),
        " (v",
        env!("CARGO_PKG_VERSION"),
        ")",
    ]
    .concat();
    easy_client.useragent(useragent.as_str()).unwrap();

    let query = args.value_of("query").unwrap();
    let mut registry = Registry::new_handle(String::from(HOST), None, easy_client);

    match registry.search(query, 10) {
        Ok((crates, _)) => {
            let json = if let Some(version) = alfred::env::version() {
                !(version.starts_with('1') || version.starts_with('2'))
            } else {
                false
            };
            workflow_output(crates, json)
        }
        Err(_) => {
            // @todo find a way in alfred to inform about the error
            workflow_output(vec![], false)
        }
    }
}

fn workflow_output(crates: Vec<Crate>, json: bool) -> Result<()> {
    let items = crates
        .into_iter()
        .map(|item| {
            let url = Cow::from(format!("{}/crates/{}", HOST, item.name));
            let description = item.description.map(Cow::from).unwrap_or_else(Cow::default);

            let docs_url = Cow::from(format!("{}/{}", DOCS, item.name));
            let docs_subtitle = format!("Docs for {}", item.name);

            alfred::ItemBuilder::new(item.name)
                .arg(url.clone())
                .arg_mod(alfred::Modifier::Command, docs_url)
                .quicklook_url(url)
                .text_large_type(description.clone())
                .subtitle(description)
                .subtitle_mod(alfred::Modifier::Command, docs_subtitle)
                .into_item()
        })
        .collect::<Vec<alfred::Item>>();

    let stdout = io::stdout().lock();
    if json {
        alfred::json::Builder::with_items(&items)
            .write(stdout)
            .wrap_err_with(|| "Couldn't write items to Alfred")?;
    } else {
        alfred::xml::write_items(stdout, &items)
            .wrap_err_with(|| "Couldn't write items to Alfred")?;
    }
    Ok(())
}
