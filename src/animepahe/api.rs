use std::sync::LazyLock;

use crate::{animepahe::unpacker, Episode, Locale, SearchResult, Server, Source};
use anyhow::Context as _;
use futures::{stream, StreamExt as _};
use kuchikiki::traits::*;
use regex::Regex;
use reqwest::{header, Client};
use serde_json::Value;
use tokio::sync::OnceCell;

async fn create_client() -> Result<Client, anyhow::Error> {
	let res = reqwest::get("https://check.ddos-guard.net/check.js").await?;
	let headers = res.headers();

	let etag_header = headers.get(header::ETAG).context("ETAG not found")?;
	let ddos_cookie = format!("__ddg2_={};", etag_header.to_str()?);

	let client = Client::builder()
		.default_headers({
			let mut headers = header::HeaderMap::new();
			headers.insert(
				header::COOKIE,
				header::HeaderValue::from_str(&ddos_cookie)?,
			);
			headers.insert(
				header::REFERER,
				header::HeaderValue::from_static("https://animepahe.ru/"),
			);
			headers.insert(
				header::USER_AGENT,
				header::HeaderValue::from_static(
					"Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.3",
				),
			);
			headers
		})
		.build()?;

	Ok(client)
}

static CLIENT: LazyLock<OnceCell<Client>> = LazyLock::new(OnceCell::new);

async fn get_client() -> Result<&'static Client, anyhow::Error> {
	Ok(CLIENT
		.get_or_init(|| async { create_client().await.unwrap() })
		.await)
}

pub async fn search(query: &str) -> Result<Vec<SearchResult>, anyhow::Error> {
	let client = get_client().await?;
	let json: Value = client
		.get(format!("https://animepahe.ru/api?m=search&q={query}"))
		.send()
		.await?
		.json()
		.await?;

	let data: Vec<SearchResult> = serde_json::from_value(json["data"].clone())?;
	Ok(data)
}

pub async fn episodes(id: &str) -> Result<Vec<Episode>, anyhow::Error> {
	let client = get_client().await?;
	let html = client
		.get(format!("https://animepahe.ru/a/{id}"))
		.send()
		.await?
		.text()
		.await?;

	let document = kuchikiki::parse_html().one(html).document_node;
	let script = document
		.select("script")
		.expect("script not found")
		.find(|x| x.text_contents().contains("let id ="))
		.context("Failed to get anime data")?
		.text_contents();

	let re = Regex::new(r#"let id = "(.*)";"#).expect("Failed to compile regex");
	let session = re
		.captures(&script)
		.context("Failed to get session")?
		.get(1)
		.context("Failed to get session")?
		.as_str();

	let client = get_client().await?;
	let json: Value = client
		.get(format!(
			"https://animepahe.ru/api?m=release&id={session}&page=1"
		))
		.send()
		.await?
		.json()
		.await?;

	let last_page = json["last_page"]
		.as_u64()
		.context("Failed to get last page")?;

	let data: Vec<Episode> = serde_json::from_value(json["data"].clone())?;
	let mut episodes = data;

	let handles = (2..=last_page)
		.map(|page_num| {
			let session = session.to_string();

			tokio::spawn(async move {
				let json: Value = client
					.get(format!(
						"https://animepahe.ru/api?m=release&id={session}&page={page_num}"
					))
					.send()
					.await?
					.json()
					.await?;

				// TODO: Make it retry on failure

				let data: Vec<Episode> = serde_json::from_value(json["data"].clone())?;
				Ok::<_, anyhow::Error>(data)
			})
		})
		.collect::<Vec<_>>();

	let results = stream::iter(handles)
		.buffer_unordered(10)
		.collect::<Vec<_>>()
		.await;

	for result in results {
		episodes.extend(result??);
	}

	episodes.sort_by_key(|episode| episode.number);

	episodes.iter_mut().for_each(|episode| {
		episode.title = format!("Episode {}", episode.number);
		episode.id = format!("{session}/{}", episode.id);
	});

	Ok(episodes)
}

pub async fn servers(ep_id: &str) -> Result<Vec<Server>, anyhow::Error> {
	let client = get_client().await?;
	let html = client
		.get(format!("https://animepahe.ru/play/{ep_id}"))
		.send()
		.await?
		.text()
		.await?;

	let document = kuchikiki::parse_html().one(html).document_node;
	let servers = document.select("#resolutionMenu button").unwrap();
	let server_list: Vec<Server> = servers
		.rev()
		.map(|server| {
			let attributes = server.attributes.borrow();
			let url = attributes.get("data-src").unwrap().to_string();
			let fansub = attributes.get("data-fansub").unwrap();
			let resolution = attributes.get("data-resolution").unwrap();
			let locale = match attributes.get("data-audio") {
				Some("eng") => Locale::Dub,
				Some("jpn") => Locale::Sub,
				_ => unimplemented!("Unknown locale"),
			};

			let name = format!("{fansub} · {resolution}p {locale}");
			Server { name, locale, url }
		})
		.collect();

	Ok(server_list)
}

pub async fn get_source(url: &str) -> Result<Source, anyhow::Error> {
	let client = get_client().await?;
	let html = client.get(url).send().await?.text().await?;
	let document = kuchikiki::parse_html().one(html).document_node;
	let script = document
		.select("script")
		.expect("script not found")
		.find(|x| x.text_contents().contains("function(p,a,c,k,e,d)"))
		.context("Failed to get video data")?
		.text_contents();

	let unpacked = unpacker::unpack_source(&script).context("Failed to unpack source")?;
	let re = Regex::new(r"https://.*\.m3u8").expect("Failed to compile regex");
	let source = re.find(&unpacked).context("Failed to get source")?.as_str();

	Ok(Source {
		url: source.to_string(),
		captions: Vec::new(),
	})
}

#[cfg(test)]
mod tests {
	use super::*;

	#[tokio::test]
	async fn test_create_client() {
		let client = create_client().await.unwrap();
		let res = client.get("https://animepahe.ru/a/4").send().await.unwrap();
		assert!(res.status().is_success());
	}

	#[tokio::test]
	async fn test_search() {
		let results = search("One Piece").await.unwrap();
		assert_eq!(results[0].id, "4");
		assert!(!results.is_empty(), "Results should not be empty");
	}

	#[tokio::test]
	async fn test_episodes() {
		let episode_list = episodes("4").await.unwrap();
		assert!(!episode_list.is_empty(), "Episode list should not be empty");
	}

	#[tokio::test]
	async fn test_servers() {
		let episode_list = episodes("4").await.unwrap();
		assert!(
			!episode_list.is_empty(),
			"Can't test servers without episodes"
		);

		let servers = servers(&episode_list[0].id).await.unwrap();
		assert_eq!(
			servers,
			vec![
				Server {
					name: "HorribleSubs · 1080p Sub".to_string(),
					locale: Locale::Sub,
					url: "https://kwik.si/e/InzZMv1U52OE".to_string(),
				},
				Server {
					name: "HorribleSubs · 720p Sub".to_string(),
					locale: Locale::Sub,
					url: "https://kwik.si/e/wkp5wNBEkkwE".to_string(),
				},
			]
		);
		assert!(!servers.is_empty(), "Server list should not be empty");
	}

	#[tokio::test]
	async fn test_get_source() {
		let source = get_source("https://kwik.si/e/InzZMv1U52OE").await.unwrap();
		assert_eq!(
			source,
			Source {
				url: "https://vault-05.padorupado.ru/stream/05/08/0df7ff5cbf5c20bf1834d37b22d918a4faa98d146dd264ce5cb83d3f30fddab6/uwu.m3u8".to_string(),
				captions: Vec::new(),
			}
		);
	}
}
