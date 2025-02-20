use crate::ciphers::rc4::{decode_url_safe_base64, rc4, replace, reverse};

pub fn decrypt(input: &str) -> String {
	let mut new_input = decode_url_safe_base64(input);
	new_input = decode_url_safe_base64(&new_input);
	new_input = rc4("E438hS1W9oRmB", &new_input);
	new_input = reverse(&new_input);
	new_input = replace(&new_input, "D5qdzkGANMQZEi", "Q5diEGMADkZzNq");
	new_input = reverse(&new_input);
	new_input = decode_url_safe_base64(&new_input);
	new_input = rc4("NZcfoMD7JpIrgQE", &new_input);
	new_input = replace(&new_input, "kTr0pjKzBqZV", "kZpjzTV0KqBr");
	new_input = decode_url_safe_base64(&new_input);
	new_input = rc4("Gay7bxj5B81TJFM", &new_input);
	new_input = replace(&new_input, "zcUxoJTi3fgyS", "oSgyJUfizcTx3");
	new_input = reverse(&new_input);
	urlencoding::decode(&new_input).unwrap().into_owned()
}