use crate::mal;
use anyhow::Context as _;
use reqwest::ClientBuilder;
use serde_json::Value;

#[derive(Debug, PartialEq)]
pub struct SkipTimes {
	pub start: f32,
	pub end: f32,
	pub skip_type: SkipType,
}

#[derive(Debug, PartialEq)]
pub enum SkipType {
	Ed,
	Op,
	Recap,
}

pub async fn get_skip_times(
	title: &str, ep_number: u16, ep_length: u32,
) -> Result<Vec<SkipTimes>, anyhow::Error> {
	let mal_id = mal::search(title).await?.id;
	let client = ClientBuilder::new().use_rustls_tls().build()?;
	let json: Value = client
		.get(format!(
			"https://api.aniskip.com/v2/skip-times/{mal_id}/{ep_number}"
		))
		.query(&[
			("types[]", "ed"),
			("types[]", "mixed-ed"),
			("types[]", "mixed-op"),
			("types[]", "op"),
			("types[]", "recap"),
			("episodeLength", &ep_length.to_string()),
		])
		.send()
		.await?
		.json()
		.await?;

	let results = json["results"].as_array().context("No results")?;
	let skip_times: Vec<SkipTimes> = results
		.iter()
		.map(|result| {
			let start = result["interval"]["startTime"].as_f64().unwrap() as f32;
			let end = result["interval"]["endTime"].as_f64().unwrap() as f32;
			let skip_type = match result["skipType"].as_str().unwrap() {
				"ed" | "mixed-ed" => SkipType::Ed,
				"op" | "mixed-op" => SkipType::Op,
				"recap" => SkipType::Recap,
				_ => unreachable!(),
			};

			SkipTimes {
				start,
				end,
				skip_type,
			}
		})
		.collect();

	Ok(skip_times)
}

#[tokio::test]
async fn test_get_skip_times() {
	let skip_times = get_skip_times("One Piece", 1, 1500).await.unwrap();
	assert_eq!(
		skip_times,
		vec![
			SkipTimes {
				start: 1387.996,
				end: 1500.0,
				skip_type: SkipType::Ed,
			},
			SkipTimes {
				start: 28.783,
				end: 118.783,
				skip_type: SkipType::Op,
			},
		]
	);
}
