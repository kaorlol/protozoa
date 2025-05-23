mod animekai;
mod animepahe;
pub mod aniskip;
mod hianime;
mod mal;

use serde::{
	de::{self, MapAccess, Visitor},
	Deserialize, Deserializer, Serialize,
};
use std::fmt;

#[derive(Debug, PartialEq, Serialize)]
pub enum Provider {
	HiAnime,
	AnimeKai,
	AnimePahe,
}

impl Provider {
	pub fn from(s: &str) -> Option<Self> {
		match s.to_lowercase().as_str() {
			"hianime" => Some(Provider::HiAnime),
			"animekai" => Some(Provider::AnimeKai),
			"animepahe" => Some(Provider::AnimePahe),
			_ => None,
		}
	}
}

impl fmt::Display for Provider {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Provider::HiAnime => write!(f, "HiAnime"),
			Provider::AnimeKai => write!(f, "AnimeKai"),
			Provider::AnimePahe => write!(f, "AnimePahe"),
		}
	}
}

pub async fn search(provider: &Provider, query: &str) -> Result<Vec<SearchResult>, anyhow::Error> {
	match provider {
		Provider::HiAnime => hianime::search(query).await,
		Provider::AnimeKai => animekai::search(query).await,
		Provider::AnimePahe => animepahe::search(query).await,
	}
}

#[derive(Clone, Debug, Serialize)]
pub struct SearchResult {
	pub title: String,
	pub poster: String,
	pub id: String,
}

impl fmt::Display for SearchResult {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", self.title)
	}
}

pub async fn episodes(provider: &Provider, id: &str) -> Result<Vec<Episode>, anyhow::Error> {
	match provider {
		Provider::HiAnime => hianime::episodes(id).await,
		Provider::AnimeKai => animekai::episodes(id).await,
		Provider::AnimePahe => animepahe::episodes(id).await,
	}
}

impl<'de> Deserialize<'de> for SearchResult {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		struct SearchResultVisitor;

		impl<'de> Visitor<'de> for SearchResultVisitor {
			type Value = SearchResult;

			fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
				formatter.write_str("a map with title, poster, and id fields")
			}

			fn visit_map<M>(self, mut map: M) -> Result<SearchResult, M::Error>
			where
				M: MapAccess<'de>,
			{
				let mut title = None;
				let mut poster = None;
				let mut id = None;

				while let Some(key) = map.next_key::<String>()? {
					match key.as_str() {
						"title" => title = Some(map.next_value()?),
						"poster" => poster = Some(map.next_value()?),
						"id" => match map.next_value::<i32>() {
							Ok(id_i32) => id = Some(id_i32.to_string()),
							Err(_) => id = Some(map.next_value()?),
						},
						_ => {
							map.next_value::<serde::de::IgnoredAny>()?;
						}
					}
				}

				let title = title.ok_or_else(|| de::Error::missing_field("title"))?;
				let poster = poster.ok_or_else(|| de::Error::missing_field("poster"))?;
				let id = id.ok_or_else(|| de::Error::missing_field("id"))?;

				Ok(SearchResult { title, poster, id })
			}
		}

		deserializer.deserialize_map(SearchResultVisitor)
	}
}

#[derive(Debug, Serialize)]
pub struct Episode {
	pub title: String,
	pub number: u32,
	pub id: String,
}

impl<'de> Deserialize<'de> for Episode {
	fn deserialize<D>(deserializer: D) -> Result<Episode, D::Error>
	where
		D: Deserializer<'de>,
	{
		struct EpisodeVisitor;

		impl<'de> Visitor<'de> for EpisodeVisitor {
			type Value = Episode;

			fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
				formatter.write_str("a map with title, number, and id fields")
			}

			fn visit_map<M>(self, mut map: M) -> Result<Episode, M::Error>
			where
				M: MapAccess<'de>,
			{
				let mut title = None;
				let mut number = None;
				let mut id = None;

				while let Some(key) = map.next_key::<String>()? {
					match key.as_str() {
						"title" => title = Some(map.next_value()?),
						"episode" => number = Some(map.next_value()?),
						"session" => id = Some(map.next_value()?),
						_ => {
							map.next_value::<serde::de::IgnoredAny>()?;
						}
					}
				}

				let title = title.ok_or_else(|| de::Error::missing_field("title"))?;
				let number = number.ok_or_else(|| de::Error::missing_field("number"))?;
				let id = id.ok_or_else(|| de::Error::missing_field("id"))?;

				Ok(Episode { title, number, id })
			}
		}

		deserializer.deserialize_map(EpisodeVisitor)
	}
}

impl fmt::Display for Episode {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", self.title)
	}
}

pub async fn servers(provider: &Provider, ep_id: &str) -> Result<Vec<Server>, anyhow::Error> {
	match provider {
		Provider::HiAnime => hianime::servers(ep_id).await,
		Provider::AnimeKai => animekai::servers(ep_id).await,
		Provider::AnimePahe => animepahe::servers(ep_id).await,
	}
}

#[derive(Debug, PartialEq, Serialize)]
pub struct Server {
	pub name: String,
	pub locale: Locale,
	pub url: String,
}

impl fmt::Display for Server {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", self.name)
	}
}

pub async fn get_source(provider: &Provider, url: &str) -> Result<Source, anyhow::Error> {
	match provider {
		Provider::HiAnime => hianime::get_source(url).await,
		Provider::AnimeKai => animekai::get_source(url).await,
		Provider::AnimePahe => animepahe::get_source(url).await,
	}
}

#[derive(Debug, Default, PartialEq, Serialize)]
pub enum Locale {
	#[default]
	HardSub,
	SoftSub,
	Dub,
	Raw,
}

impl fmt::Display for Locale {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Locale::HardSub => write!(f, "HardSub"),
			Locale::SoftSub => write!(f, "SoftSub"),
			Locale::Dub => write!(f, "Dub"),
			Locale::Raw => write!(f, "Raw"),
		}
	}
}

#[derive(Debug, PartialEq, Serialize)]
pub struct Source {
	pub url: String,
	pub captions: Vec<Caption>,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct Caption {
	#[serde(rename = "file")]
	pub url: String,
	pub label: Option<String>,
	pub kind: String,
}

impl fmt::Display for Caption {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", self.label.as_ref().unwrap())
	}
}
