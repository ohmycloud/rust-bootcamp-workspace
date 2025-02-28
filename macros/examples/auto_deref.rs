use macros::AutoDeref;

#[allow(unused)]
#[derive(Debug, AutoDeref)]
#[deref(field = "inner")]
pub struct RespBulkString {
    inner: String,
    nothing: (),
}
fn main() {
    let s = RespBulkString {
        inner: "Hello".to_string(),
        nothing: (),
    };
    println!("{:?}", s);
}
