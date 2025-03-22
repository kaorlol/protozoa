use crate::ciphers::rc4::{decode_url_safe_base64, rc4, replace, reverse, url_safe_base64};

pub fn encrypt(input: &str) -> String {
	let text = urlencoding::encode(input).into_owned();
	url_safe_base64(&replace(
		&url_safe_base64(&rc4(
			"sXmH96C4vhRrgi8",
			&reverse(&reverse(&url_safe_base64(&rc4(
				"kOCJnByYmfI",
				&replace(
					&replace(
						&reverse(&url_safe_base64(&rc4("0DU8ksIVlFcia2", &text))),
						"1wctXeHqb2",
						"1tecHq2Xbw",
					),
					"48KbrZx1ml",
					"Km8Zb4lxr1",
				),
			)))),
		)),
		"hTn79AMjduR5",
		"djn5uT7AMR9h",
	))
}

pub fn decrypt(input: &str) -> String {
	let text = rc4(
		"0DU8ksIVlFcia2",
		&decode_url_safe_base64(&reverse(&replace(
			&replace(
				&rc4(
					"kOCJnByYmfI",
					&decode_url_safe_base64(&reverse(&reverse(&rc4(
						"sXmH96C4vhRrgi8",
						&decode_url_safe_base64(&replace(
							&decode_url_safe_base64(input),
							"djn5uT7AMR9h",
							"hTn79AMjduR5",
						)),
					)))),
				),
				"Km8Zb4lxr1",
				"48KbrZx1ml",
			),
			"1tecHq2Xbw",
			"1wctXeHqb2",
		))),
	);
	urlencoding::decode(&text).unwrap().into_owned()
}

#[test]
fn test_decrypt() {
	let decrypted = decrypt("V3BFMUdfaXM4cU5fTnNodWwxNTE5b3FRM3pGQWltMlBzNHRLQlNPMDQyclh1T0p3N1dGelhIUWZzQUE4U1ltbjFwZzVXOXFrMVpSeVE1YWFJeTZUYU9kUDRxbDRjb1RZM2pWc3JzVVQ4VEo0TUw2YXROemwweTJkUkM1SnlRNDB3MG13dF9CVjZ0VF9EcFFtQy0zUThoazY1T0FiNmQ4MjNjblc1VGJYVFhhRUxCNzRNX21NalYxMGotaG5ZRW5ZaHY0bThvdFkwejI0Y3N1d0laU0pjWG9nZUd2NzJ3VnFRcjlaTDM3cDJhMHo3ODdmWVNjZkw4TEpwanlqbUhHUDJrSkwybnN3MGF1ZUo4NU5pc0hWRFZxT2pvejNHQ0hrUEVBS19SZVNXb25iakZzRkFTM3Fwd2R2R1NKVUk4eXk2cC1yOGZ3b2xlZVc4NjdMZTdYTEZ2cHJnd09tTW4ycGZZQWhyZUltX09DNFFRMHh2d0RZajhRTklqQlQxSTl1ZENXTTB2N092OGtuMzdmeVpkaFM1Qy1ZMUpmVnNoeDdaanM");
	println!("{}", decrypted);
}

#[test]
fn test_encrypt() {
	let encrypted = encrypt(r#"{"url":"https:\/\/megaup.cc\/e\/2MivLzL-WS2JcOLxE7xN6hfpCQ","skip":{"intro":[91,180],"outro":[1325,1414]}}"#);
	println!("{}", encrypted);
}