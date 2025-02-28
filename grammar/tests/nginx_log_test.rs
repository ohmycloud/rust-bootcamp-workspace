use log_parser::parser::nginx_log::take_list;

#[test]
fn test_parse_ip() {
    let mut input = "93.180.71.3";
    if let Ok(ip) = take_list(&mut input) {
        assert_eq!("93.180.71.3", ip);
    }
}
