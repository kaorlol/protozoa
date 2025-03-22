use crate::{Caption, Episode, Locale, SearchResult, Server, Source};
use anyhow::Context as _;
use kuchikiki::traits::*;
use protozoa_cryptography::sources::megacloud;
use serde_json::Value;

pub async fn search(query: &str) -> Result<Vec<SearchResult>, anyhow::Error> {
	let html = reqwest::get(format!("https://hianime.to/search?keyword={query}"))
		.await?
		.text()
		.await?;

	let document = kuchikiki::parse_html().one(html);
	let items = document
		.select(".flw-item")
		.map_err(|_| anyhow::anyhow!("Failed to select items"))?;

	let results: Vec<SearchResult> = items
		.map(|item| {
			let film_poster = item.as_node().select_first(".film-poster-img").unwrap();
			let item_qtip = item.as_node().select_first(".item-qtip").unwrap();

			let attributes = item_qtip.attributes.borrow();
			let title = attributes.get("title").unwrap();
			let id = attributes.get("data-id").unwrap();

			let attributes = film_poster.attributes.borrow();
			let poster = attributes.get("data-src").unwrap();

			SearchResult {
				title: title.to_string(),
				poster: poster.to_string(),
				id: id.to_string(),
			}
		})
		.collect();

	Ok(results)
}

pub async fn episodes(id: &str) -> Result<Vec<Episode>, anyhow::Error> {
	let json: Value = reqwest::get(format!("https://hianime.to/ajax/v2/episode/list/{id}"))
		.await?
		.json()
		.await?;

	let html = json["html"].as_str().unwrap();
	let document = kuchikiki::parse_html().one(html);
	let episodes = document
		.select(".ep-item")
		.map_err(|_| anyhow::anyhow!("Failed to select episodes"))?;

	let episode_list: Vec<Episode> = episodes
		.map(|episode| {
			let attributes = episode.attributes.borrow();

			let title = attributes.get("title").unwrap().replace("&#39;", "'");
			let id = attributes.get("data-id").unwrap().to_string();
			let number = attributes.get("data-number").unwrap().parse().unwrap();

			Episode { title, id, number }
		})
		.collect();

	Ok(episode_list)
}

pub async fn servers(ep_id: &str) -> Result<Vec<Server>, anyhow::Error> {
	let json: Value = reqwest::get(format!(
		"https://hianime.to/ajax/v2/episode/servers?episodeId={ep_id}"
	))
	.await?
	.json()
	.await?;

	let html = json["html"].as_str().unwrap();
	let document = kuchikiki::parse_html().one(html);
	let servers = document
		.select(".server-item")
		.map_err(|_| anyhow::anyhow!("Failed to select servers"))?;

	let mut server_list = Vec::new();

	for server in servers {
		let attributes = server.attributes.borrow();
		let name = server.text_contents();
		let server_id = attributes.get("data-id").unwrap();
		let locale = match attributes.get("data-type") {
			Some("sub") => Locale::SoftSub,
			Some("dub") => Locale::Dub,
			Some("raw") => Locale::Raw,
			_ => unimplemented!("Unknown locale"),
		};

		let json: Value = reqwest::get(format!(
			"https://hianime.to/ajax/v2/episode/sources?id={server_id}"
		))
		.await?
		.json()
		.await?;

		let url = json["link"].as_str().unwrap().to_string();

		let name = format!("{} · {locale}", name.trim());
		server_list.push(Server { name, locale, url });
	}

	Ok(server_list)
}

pub async fn get_source(url: &str) -> Result<Source, anyhow::Error> {
	let xrax = url.rsplit_once('/').unwrap().1.split('?').next().unwrap();
	let (json, secret) = megacloud::get_sources(xrax.to_string()).await?;

	let json: Value = serde_json::from_str(&json).context("Failed to parse sources")?;
	let cipher_text = json["sources"].as_str().unwrap();
	let decrypted = megacloud::decrypt(cipher_text, &secret)?;

	let sources: Vec<Value> =
		serde_json::from_str(&decrypted).context("Failed to parse decrypted")?;
	let url = sources[0]["file"].as_str().unwrap().to_string();
	let mut captions: Vec<Caption> = serde_json::from_value(json["tracks"].clone())?;
	captions.retain(|track| track.kind != "thumbnails");

	Ok(Source { url, captions })
}

#[cfg(test)]
mod tests {
	use super::*;

	#[tokio::test]
	async fn test_search() {
		let results = search("One Piece").await.unwrap();
		assert_eq!(results[1].id, "100");
		assert!(!results.is_empty(), "Results should not be empty");
	}

	#[tokio::test]
	async fn test_episodes() {
		let episode_list = episodes("100").await.unwrap();
		assert!(!episode_list.is_empty(), "Episode list should not be empty");
	}

	#[tokio::test]
	async fn test_servers() {
		let servers = servers("2142").await.unwrap();
		assert!(!servers.is_empty(), "Servers should not be empty");
	}

	#[tokio::test]
	async fn test_get_source() {
		let servers = servers("2142").await.unwrap();
		assert!(!servers.is_empty(), "Can't test source without servers");

		let source = get_source(&servers[0].url).await.unwrap();
		assert!(!source.url.is_empty(), "Source url should not be empty");
	}
}
