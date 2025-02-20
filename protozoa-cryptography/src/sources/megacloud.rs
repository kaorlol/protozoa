use aes::cipher::{generic_array::GenericArray, KeyIvInit};
use anyhow::Context as _;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use cbc::{
	cipher::{block_padding::Pkcs7, BlockDecryptMut},
	Decryptor,
};
use regex::Regex;
use reqwest::{header, ClientBuilder};
use rustyscript::{json_args, Module, ModuleWrapper};

async fn get_wasm() -> Result<Vec<u8>, anyhow::Error> {
	let client = ClientBuilder::new()
		.default_headers({
			let mut headers = header::HeaderMap::new();
			headers.insert(
				header::REFERER,
				header::HeaderValue::from_static("https://hianime.to"),
			);
			headers
		})
		.build()?;

	let res = client
		.get("https://megacloud.tv/images/loading.png?v=0.0.9")
		.send()
		.await?
		.bytes()
		.await?;

	Ok(res.to_vec())
}

async fn get_meta(xrax: &str) -> Result<String, anyhow::Error> {
	let client = ClientBuilder::new()
		.default_headers({
			let mut headers = header::HeaderMap::new();
			headers.insert(
				header::REFERER,
				header::HeaderValue::from_static("https://hianime.to"),
			);
			headers
		})
		.build()?;

	let html = client
		.get(format!("https://megacloud.tv/embed-2/e-1/{xrax}"))
		.send()
		.await?
		.text()
		.await?;

	let meta = Regex::new(r#"<meta name="j_crt" content="(.+?)">"#)?;
	let content = &meta.captures(&html).context("Failed to get meta")?[1];

	Ok(content.to_string())
}

#[derive(Debug)]
pub struct Rabbit {
	pub secret: String,
	pub pid: String,
	pub kversion: String,
	pub kid: String,
	pub browser_version: String,
}

async fn rabbit(xrax: &str) -> Result<Rabbit, anyhow::Error> {
	let meta = get_meta(xrax).await?;
	let wasm = get_wasm().await?;

	let script = reqwest::get(
		"https://raw.githubusercontent.com/kaorlol/protozoa/refs/heads/main/protozoa-cryptography/rabbit.js",
	)
	.await?
	.bytes()
	.await?
	.to_vec();

	let xrax = xrax.to_string();
	let result = tokio::task::spawn_blocking(move || -> Result<Rabbit, anyhow::Error> {
		let module = Module::new("rabbit.js", String::from_utf8_lossy(&script));

		let mut module_wrapper = ModuleWrapper::new_from_module(&module, Default::default())?;

		let values: (String, String, String, String, String) =
			module_wrapper.call("get_args", json_args!(xrax, meta, wasm))?;

		Ok(Rabbit {
			secret: values.0,
			pid: values.1,
			kversion: values.2,
			kid: values.3,
			browser_version: values.4,
		})
	});

	result.await?
}

pub async fn get_sources(xrax: String) -> Result<(String, String), anyhow::Error> {
	let rab = rabbit(&xrax).await?;

	let client = ClientBuilder::new()
		.default_headers({
			let mut headers = header::HeaderMap::new();
			headers.insert(
				header::REFERER,
				header::HeaderValue::from_str(&format!("https://megacloud.tv/embed-2/e-1/{xrax}"))
					.context("Failed to create referer")?,
			);
			headers.insert(
				header::USER_AGENT,
				header::HeaderValue::from_static(
					"Mozilla/5.0 (X11; Linux x86_64; rv:133.0) Gecko/20100101 Firefox/133.0",
				),
			);
			headers.insert(
				"X-Requested-With",
				header::HeaderValue::from_static("XMLHttpRequest"),
			);
			headers
		})
		.build()?;

	let url = format!(
		"https://megacloud.tv/embed-2/ajax/e-1/getSources?id={}&v={}&h={}&b={}",
		rab.pid, rab.kversion, rab.kid, rab.browser_version
	);

	let text = client.get(&url).send().await?.text().await?;
	Ok((text, rab.secret))
}

fn generate_encryption_key(salt: &[u8], secret_bytes: &[u8]) -> Vec<u8> {
	let mut key = md5::compute([secret_bytes, salt].concat()).to_vec();
	let mut current_key = key.clone();

	while current_key.len() < 48 {
		key = md5::compute([key, secret_bytes.to_vec(), salt.to_vec()].concat()).to_vec();
		current_key.extend_from_slice(&key);
	}

	current_key
}

type Aes256CbcDec = Decryptor<aes::Aes256Dec>;

fn decrypt_aes(ciphertext: &[u8], key: &[u8]) -> String {
	let iv = &key[32..];
	let key = &key[..32];

	let mut buf = ciphertext.to_vec();
	let pt = Aes256CbcDec::new(GenericArray::from_slice(key), GenericArray::from_slice(iv))
		.decrypt_padded_mut::<Pkcs7>(&mut buf)
		.unwrap();

	String::from_utf8_lossy(pt).to_string()
}

pub fn decrypt(ciphertext_b64: &str, secret: &str) -> Result<String, anyhow::Error> {
	let encrypted_data = STANDARD.decode(ciphertext_b64)?;

	let salt = &encrypted_data[8..16];
	let encrypted_payload = &encrypted_data[16..];

	let key = generate_encryption_key(salt, secret.as_bytes());
	let decrypted_text = decrypt_aes(encrypted_payload, &key);

	Ok(decrypted_text)
}
