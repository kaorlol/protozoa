use crate::ciphers::rc4::{decode_url_safe_base64, rc4, replace, reverse, url_safe_base64};

pub fn encrypt(input: &str) -> String {
	let mut text = urlencoding::encode(input).into_owned();
	text = reverse(&text);
	text = rc4("gEUzYavPrGpj", &text);
	text = url_safe_base64(&text);
	text = replace(&text, "U8nv0tEFGTb", "bnGvE80UtTF");
	text = replace(&text, "9ysoRqBZHV", "oqsZyVHBR9");
	text = rc4("CSk63F7PwBHJKa", &text);
	text = url_safe_base64(&text);
	text = reverse(&text);
	text = replace(&text, "cKj9BMN15LsdH", "NL5cdKs1jB9MH");
	text = rc4("T2zEp1WHL9CsSk7", &text);
	text = url_safe_base64(&text);
	text = reverse(&text);
	url_safe_base64(&text)
}

pub fn decrypt(input: &str) -> String {
	let mut new_input = decode_url_safe_base64(input);
	new_input = reverse(&new_input);
	new_input = decode_url_safe_base64(&new_input);
	new_input = rc4("T2zEp1WHL9CsSk7", &new_input);
	new_input = replace(&new_input, "NL5cdKs1jB9MH", "cKj9BMN15LsdH");
	new_input = reverse(&new_input);
	new_input = decode_url_safe_base64(&new_input);
	new_input = rc4("CSk63F7PwBHJKa", &new_input);
	new_input = replace(&new_input, "oqsZyVHBR9", "9ysoRqBZHV");
	new_input = replace(&new_input, "bnGvE80UtTF", "U8nv0tEFGTb");
	new_input = decode_url_safe_base64(&new_input);
	new_input = rc4("gEUzYavPrGpj", &new_input);
	new_input = reverse(&new_input);
	urlencoding::decode(&new_input).unwrap().into_owned()
}
