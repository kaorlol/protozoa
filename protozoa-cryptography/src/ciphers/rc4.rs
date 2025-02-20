use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};

pub fn rc4(key: &str, data: &str) -> String {
	let mut s: Vec<u8> = (0..=255).collect();
	let mut j: usize = 0;

	let key_utf16: Vec<u16> = key.encode_utf16().collect();

	for i in 0..256 {
		j = (j + s[i] as usize + key_utf16[i % key_utf16.len()] as usize) % 256;
		s.swap(i, j);
	}

	let mut i: usize = 0;
	j = 0;
	let mut res = Vec::with_capacity(data.len() * 2);
	let data_utf16: Vec<u16> = data.encode_utf16().collect();

	for &byte in data_utf16.iter() {
		i = (i + 1) % 256;
		j = (j + s[i] as usize) % 256;
		s.swap(i, j);
		let k = s[(s[i] as usize + s[j] as usize) % 256];
		res.push((byte ^ k as u16) as u8);
	}

	String::from_utf16_lossy(&res.iter().map(|&b| b as u16).collect::<Vec<_>>())
}

pub fn reverse(input: &str) -> String {
	input.chars().rev().collect()
}

pub fn url_safe_base64(input: &str) -> String {
	let mut byte_vec: Vec<u8> = Vec::new();
	for char in input.chars() {
		let byte = char as u8;
		byte_vec.push(byte);
	}
	URL_SAFE_NO_PAD.encode(byte_vec)
}

pub fn decode_url_safe_base64(input: &str) -> String {
	let mut byte_vec = URL_SAFE_NO_PAD.decode(input).unwrap();
	let mut char_vec: Vec<char> = Vec::new();
	for byte in byte_vec.drain(..) {
		char_vec.push(byte as char);
	}
	char_vec.into_iter().collect()
}

pub fn replace(input: &str, search_chars: &str, replace_chars: &str) -> String {
	let mut replacement_map = std::collections::HashMap::new();

	for (s, r) in search_chars.chars().zip(replace_chars.chars()) {
		replacement_map.insert(s, r);
	}

	input
		.chars()
		.map(|c| replacement_map.get(&c).copied().unwrap_or(c))
		.collect()
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_rc4() {
		let test = rc4("key", "Plaintext");
		println!("{}", test);
		assert_eq!(test, "[\0U\u{84}Jû\u{1e}2<");
	}

	#[test]
	fn test_reverse() {
		let test = reverse("Hello, World!");
		assert_eq!(test, "!dlroW ,olleH");
	}

	#[test]
	fn test_url_safe_base64() {
		let test = url_safe_base64("Hello, World!");
		assert_eq!(test, "SGVsbG8sIFdvcmxkIQ");
	}

	#[test]
	fn test_decode_url_safe_base64() {
		let test = decode_url_safe_base64("SGVsbG8sIFdvcmxkIQ");
		assert_eq!(test, "Hello, World!");
	}

	#[test]
	fn test_replace() {
		let test = replace("Hello, World!", "HW", "hw");
		assert_eq!(test, "hello, world!");
	}
}
