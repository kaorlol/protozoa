mod animekai;
mod animepahe;
mod hianime;

use serde::{
	de::{self, MapAccess, Visitor},
	Deserialize, Deserializer,
};
use std::fmt;

pub enum Provider {
	HiAnime,
	AnimeKai,
	AnimePahe,
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
		Provider::HiAnime => hianime::ajax::search(query).await,
		Provider::AnimeKai => animekai::ajax::search(query).await,
		Provider::AnimePahe => animepahe::api::search(query).await,
	}
}

#[derive(Clone, Debug)]
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

impl SearchResult {
	pub async fn episodes(&self, provider: &Provider) -> Result<Vec<Episode>, anyhow::Error> {
		match provider {
			Provider::HiAnime => hianime::ajax::episodes(&self.id).await,
			Provider::AnimeKai => animekai::ajax::episodes(&self.id).await,
			Provider::AnimePahe => animepahe::api::episodes(&self.id).await,
		}
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

#[derive(Debug, Deserialize)]
pub struct Episode {
	pub title: String,
	#[serde(alias = "episode")]
	pub number: u32,
	#[serde(rename = "session")]
	pub id: String,
}

impl fmt::Display for Episode {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", self.title)
	}
}

impl Episode {
	pub async fn servers(&self, provider: &Provider) -> Result<Vec<Server>, anyhow::Error> {
		match provider {
			Provider::HiAnime => hianime::ajax::servers(&self.id).await,
			Provider::AnimeKai => animekai::ajax::servers(&self.id).await,
			Provider::AnimePahe => animepahe::api::servers(&self.id).await,
		}
	}
}

#[derive(Debug, PartialEq)]
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

impl Server {
	pub async fn get_source(&self, provider: &Provider) -> Result<Source, anyhow::Error> {
		match provider {
			Provider::HiAnime => hianime::ajax::get_source(&self.url).await,
			Provider::AnimeKai => animekai::ajax::get_source(&self.url).await,
			Provider::AnimePahe => animepahe::api::get_source(&self.url).await,
		}
	}
}

#[derive(Debug, Default, PartialEq)]
pub enum Locale {
	#[default]
	Sub,
	Dub,
	Raw,
}

impl fmt::Display for Locale {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Locale::Sub => write!(f, "Sub"),
			Locale::Dub => write!(f, "Dub"),
			Locale::Raw => write!(f, "Raw"),
		}
	}
}

#[derive(Debug, PartialEq)]
pub struct Source {
	pub url: String,
	pub captions: Vec<Caption>,
}

#[derive(Debug, Deserialize, PartialEq)]
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
