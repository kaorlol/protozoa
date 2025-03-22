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
	let decrypted = decrypt("UVJNWkZQbWl0WnRfN0lVdUFXajFKYXE5enpCcTN6Nm9rcTc5UW1ta1JIeldoUnBjYkJFUmNHME9rSFBfVzZTSTY0VUpaZHFOcFo2dFVLV19lQ2lUWmVnVmwtWTNDS0kxeHlWcmxPbzV0UUo0ajVMeXJMclRyQTZiMURieHJBd1p5MmZOdl9KRGs0bzhWYVVyQ3VYeTZoeDc1T2ZKX2dUbzJOTE04a3JkTzJHSEtkZzVjWV9JR2xCblM4QjVYR1BkLThZSDY4cFloU0stWGM0ZElaNk5hRmN2QzBuRW9DQkU5WklISzN3b3dhSXVIOHVXWW5FamN2ZnNwZ3pFZG1INDN5TUg4VzdpNDV1UE5fQUptN2Z3YlYtdEZLbm83RmZ2SWtXNndmQ0JMZnJEamQ5NUFRamJvUTdySTBodlBJRzBocnJ5MnZ3aHAtLWFuSzd0ZUxmTDlMWkRwV0NuWlV2RlBLamw2UGdobk1iMGFDTkhzd0RFTk5va0J3bEN4YTFIMDM2Qm92RkN5UnBod19iaE1WZEZzQnJ2Mk9QcWlzTm9aZFFD");
	println!("{}", decrypted);
}

#[test]
fn test_encrypt() {
	let encrypted = encrypt(r#"{"url":"https:\/\/megaup.cc\/e\/2MivLzL-WS2JcOLxE7xN6hfpCQ","skip":{"intro":[91,180],"outro":[1325,1414]}}"#);
	println!("{}", encrypted);
}