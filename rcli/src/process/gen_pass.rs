use rand::seq::SliceRandom;

const UPPERCASE: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ";
const LOWERCASE: &[u8] = b"abcdefghijklmnpqrstuvwxyz";
const NUMBER: &[u8] = b"123456789";
const SYMBOL: &[u8] = b"!@#$%^&*_";
pub fn process_genpass(
    length: u8,
    lc: bool,
    uc: bool,
    number: bool,
    symbol: bool,
) -> anyhow::Result<String> {
    let mut rng = rand::thread_rng();
    let mut password = Vec::new();
    let mut chars = Vec::new();

    if uc {
        chars.extend_from_slice(UPPERCASE);
        password.push(*UPPERCASE.choose(&mut rng).expect("Expect a uppercase"));
    }
    if lc {
        chars.extend_from_slice(LOWERCASE);
        password.push(*LOWERCASE.choose(&mut rng).expect("Expect a lowercase"));
    }

    if number {
        chars.extend_from_slice(NUMBER);
        password.push(*NUMBER.choose(&mut rng).expect("Expect a number"));
    }

    if symbol {
        chars.extend_from_slice(SYMBOL);
        password.push(*SYMBOL.choose(&mut rng).expect("Expect a symbol"));
    }

    for _ in 0..(length - password.len() as u8) {
        let c = chars.choose(&mut rng).expect("Chars must not be empty");
        password.push(*c);
    }
    password.shuffle(&mut rng);

    Ok(String::from_utf8(password).expect("Can't generate a random password"))
}
