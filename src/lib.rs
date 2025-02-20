mod animekai;
mod animepahe;
mod hianime;

use serde::{
	de::{self, MapAccess, Visitor},
	Deserialize, Deserializer,
};
use std::{cmp::min, fmt, str::Chars};

pub enum Provider {
	HiAnime,
	AnimeKai,
	AnimePahe,
}

pub async fn search(provider: Provider, query: &str) -> Result<SearchResults, anyhow::Error> {
	match provider {
		Provider::HiAnime => hianime::ajax::search(query).await,
		Provider::AnimeKai => animekai::ajax::search(query).await,
		Provider::AnimePahe => animepahe::api::search(query).await,
	}
}

#[derive(Debug)]
pub struct SearchResults {
	pub closest_match: SearchResult,
	pub results: Vec<SearchResult>,
}

#[derive(Clone, Debug)]
pub struct SearchResult {
	pub title: String,
	pub poster: String,
	pub id: String,
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

#[derive(Debug, PartialEq)]
pub struct Server {
	pub name: String,
	pub locale: Locale,
	pub url: String,
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

// https://github.com/rapidfuzz/strsim-rs/blob/main/src/lib.rs#L166
struct StringWrapper<'a>(&'a str);

impl<'b> IntoIterator for &StringWrapper<'b> {
	type Item = char;
	type IntoIter = Chars<'b>;

	fn into_iter(self) -> Self::IntoIter {
		self.0.chars()
	}
}

/// https://github.com/rapidfuzz/strsim-rs/blob/main/src/lib.rs#L233
fn generic_levenshtein<'a, 'b, Iter1, Iter2, Elem1, Elem2>(a: &'a Iter1, b: &'b Iter2) -> usize
where
	&'a Iter1: IntoIterator<Item = Elem1>,
	&'b Iter2: IntoIterator<Item = Elem2>,
	Elem1: PartialEq<Elem2>,
{
	let b_len = b.into_iter().count();

	let mut cache: Vec<usize> = (1..b_len + 1).collect();

	let mut result = b_len;

	for (i, a_elem) in a.into_iter().enumerate() {
		result = i + 1;
		let mut distance_b = i;

		for (j, b_elem) in b.into_iter().enumerate() {
			let cost = usize::from(a_elem != b_elem);
			let distance_a = distance_b + cost;
			distance_b = cache[j];
			result = min(result + 1, min(distance_a, distance_b + 1));
			cache[j] = result;
		}
	}

	result
}

/// https://github.com/rapidfuzz/strsim-rs/blob/main/src/lib.rs#L269
fn levenshtein(a: &str, b: &str) -> usize {
	generic_levenshtein(&StringWrapper(a), &StringWrapper(b))
}

/// https://github.com/rapidfuzz/strsim-rs/blob/main/src/lib.rs#L285
fn normalized_levenshtein(a: &str, b: &str) -> f64 {
	if a.is_empty() && b.is_empty() {
		return 1.0;
	}
	1.0 - (levenshtein(a, b) as f64) / (a.chars().count().max(b.chars().count()) as f64)
}

pub fn get_closest_match<'a>(query: &str, results: &'a [SearchResult]) -> Option<&'a SearchResult> {
	results.iter().max_by(|a, b| {
		normalized_levenshtein(query, &a.title)
			.partial_cmp(&normalized_levenshtein(query, &b.title))
			.unwrap()
	})
}
