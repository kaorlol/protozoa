use crate::{Caption, Episode, Locale, SearchResult, Server, Source};
use anyhow::Context as _;
use kuchikiki::traits::*;
use protozoa_cryptography::sources::{animekai, megaup};
use serde_json::Value;

pub async fn search(query: &str) -> Result<Vec<SearchResult>, anyhow::Error> {
	let json: Value = reqwest::get(format!(
		"https://animekai.to/ajax/anime/search?keyword={query}"
	))
	.await?
	.json()
	.await?;

	let html = json["result"]["html"].as_str().context("No result")?;
	let document = kuchikiki::parse_html().one(html);

	let items = document
		.select(".aitem")
		.map_err(|_| anyhow::anyhow!("No items"))?;

	let results: Vec<SearchResult> = items
		.map(|item| {
			let attributes = item.attributes.borrow();
			let id = attributes.get("href").unwrap().rsplit_once('-').unwrap().1;

			let poster_img = item.as_node().select_first("img").unwrap();
			let attributes = poster_img.attributes.borrow();
			let poster = attributes.get("src").unwrap();

			let title = item
				.as_node()
				.select_first(".title")
				.unwrap()
				.text_contents();

			SearchResult {
				title,
				poster: poster.to_string(),
				id: id.to_string(),
			}
		})
		.collect();

	Ok(results)
}

pub async fn episodes(id: &str) -> Result<Vec<Episode>, anyhow::Error> {
	let html = reqwest::get(format!("https://animekai.to/watch/{id}"))
		.await?
		.text()
		.await?;

	let document = kuchikiki::parse_html().one(html);
	let bookmark = document
		.select_first(".user-bookmark")
		.map_err(|_| anyhow::anyhow!("No bookmark"))?;

	let bookmark_id = {
		let attributes = bookmark.attributes.borrow();
		attributes.get("data-id").unwrap().to_string()
	};

	let enc_id = animekai::encrypt(&bookmark_id);

	let json: Value = reqwest::get(format!(
		"https://animekai.to/ajax/episodes/list?ani_id={bookmark_id}&_={enc_id}"
	))
	.await?
	.json()
	.await?;

	let html = json["result"].as_str().context("No result")?;
	let document = kuchikiki::parse_html().one(html);
	let episodes = document
		.select("a")
		.map_err(|_| anyhow::anyhow!("No episodes"))?;

	let episode_list = episodes
		.map(|episode| {
			let attributes = episode.attributes.borrow();
			let id = attributes.get("token").context("No token")?;
			let title = episode
				.as_node()
				.select_first("span")
				.map_err(|_| anyhow::anyhow!("No title"))?
				.text_contents();

			let number = attributes.get("num").unwrap().parse().unwrap();

			Ok(Episode {
				id: id.to_string(),
				title,
				number,
			})
		})
		.collect::<Result<Vec<Episode>, anyhow::Error>>()?;

	Ok(episode_list)
}

pub async fn servers(token: &str) -> Result<Vec<Server>, anyhow::Error> {
	let enc_token = animekai::encrypt(token);

	let json: Value = reqwest::get(format!(
		"https://animekai.to/ajax/links/list?token={token}&_={enc_token}"
	))
	.await?
	.json()
	.await?;

	let html = json["result"].as_str().context("No result")?;

	let document = kuchikiki::parse_html().one(html);
	let servers = document
		.select(".server")
		.map_err(|_| anyhow::anyhow!("No servers"))?;

	let mut server_list = Vec::new();
	for server in servers {
		let attributes = server.attributes.borrow();
		let name = server.text_contents();
		let tid = attributes.get("data-tid").unwrap();
		let locale = match tid.rsplit_once("_").unwrap().1 {
			"sub" => Locale::HardSub,
			"dub" => Locale::Dub,
			"softsub" => Locale::SoftSub,
			_ => unimplemented!("Unknown locale"),
		};

		let lid = attributes.get("data-lid").unwrap();
		let enc_lid = animekai::encrypt(lid);

		let json: Value = reqwest::get(format!(
			"https://animekai.to/ajax/links/view?id={lid}&_={enc_lid}"
		))
		.await?
		.json()
		.await?;

		let result = json["result"].as_str().context("No result")?;
		let json: Value = serde_json::from_str(&animekai::decrypt(result))?;
		let url = json["url"].as_str().context("No url")?.to_string();

		let name = format!("{name} · {locale}");

		server_list.push(Server { name, url, locale });
	}

	Ok(server_list)
}

pub async fn get_source(url: &str) -> Result<Source, anyhow::Error> {
	let json: Value = reqwest::get(url.replace("/e/", "/media/"))
		.await?
		.json()
		.await?;

	let result = json["result"].as_str().context("No result")?;
	let decrypted = megaup::decrypt(result);
	let json: Value = serde_json::from_str(&decrypted)?;

	let url = json["sources"][0]["file"].as_str().context("No file")?;
	let mut captions: Vec<Caption> = serde_json::from_value(json["tracks"].clone())?;
	captions.retain(|caption| caption.kind != "thumbnails");

	Ok(Source {
		url: url.to_string(),
		captions,
	})
}

#[cfg(test)]
mod tests {
	use super::*;

	#[tokio::test]
	async fn test_search() {
		let results = search("One Piece").await.unwrap();
		assert_eq!(results[0].id, "dk6r");
		assert!(!results.is_empty(), "Results should not be empty");
	}

	#[tokio::test]
	async fn test_episodes() {
		let episodes = episodes("dk6r").await.unwrap();
		assert!(!episodes.is_empty(), "Episodes should not be empty");
	}

	#[tokio::test]
	async fn test_servers() {
		let servers = servers("ccTwp_Hxokjv02gVx4if").await.unwrap();
		assert!(!servers.is_empty(), "Servers should not be empty");
	}

	#[tokio::test]
	async fn test_get_source() {
		let servers = servers("ccTwp_Hxokjv02gVx4if").await.unwrap();
		assert!(!servers.is_empty(), "Can't test source without servers");

		println!("{:#?}", servers);

		let source = get_source(&servers[0].url).await.unwrap();
		assert!(!source.url.is_empty(), "Source url should not be empty");
	}
}
