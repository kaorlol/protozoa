use std::{cmp::min, str::Chars};

use anyhow::Context as _;
use serde_json::Value;

#[derive(Clone, Debug, PartialEq)]
pub struct SearchResult {
	pub title: String,
	pub id: String,
}

pub async fn search(query: &str) -> Result<SearchResult, anyhow::Error> {
	let json: Value = reqwest::get(format!(
		"https://myanimelist.net/search/prefix.json?type=anime&keyword={query}"
	))
	.await?
	.json()
	.await?;

	let items = json["categories"][0]["items"]
		.as_array()
		.context("No items")?;

	let results: Vec<SearchResult> = items
		.iter()
		.map(|item| {
			let title = item["name"].as_str().unwrap();
			let id = item["id"].as_u64().unwrap();

			SearchResult {
				title: title.to_string(),
				id: id.to_string(),
			}
		})
		.collect();

	let best = results
		.iter()
		.max_by(|a, b|{
			normalized_levenshtein(&a.title, query)
				.partial_cmp(&normalized_levenshtein(&b.title, query))
				.unwrap()
		})
		.context("No results")?;

	Ok(best.clone())
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

// https://github.com/rapidfuzz/strsim-rs/blob/main/src/lib.rs#L233
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

// https://github.com/rapidfuzz/strsim-rs/blob/main/src/lib.rs#L269
fn levenshtein(a: &str, b: &str) -> usize {
	generic_levenshtein(&StringWrapper(a), &StringWrapper(b))
}

// https://github.com/rapidfuzz/strsim-rs/blob/main/src/lib.rs#L285
fn normalized_levenshtein(a: &str, b: &str) -> f64 {
	if a.is_empty() && b.is_empty() {
		return 1.0;
	}
	1.0 - (levenshtein(a, b) as f64) / (a.chars().count().max(b.chars().count()) as f64)
}

#[tokio::test]
async fn test_search() {
	let result = search("One Piece").await.unwrap();
	assert_eq!(
		result,
		SearchResult {
			title: "One Piece".to_string(),
			id: "21".to_string(),
		}
	);
}
