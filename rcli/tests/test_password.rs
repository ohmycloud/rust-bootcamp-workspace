use rcli::process::process_genpass;
use zxcvbn::zxcvbn;

#[test]
fn test_password() {
    let password = process_genpass(12, true, true, true, true);
    let estimate = zxcvbn(password.unwrap().as_str(), &[]).unwrap();
    assert_eq!(estimate.score(), 4);
}
