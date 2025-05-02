use anyhow::Result;
use bytes::{BufMut, BytesMut};

fn main() -> Result<()> {
    let mut buf = BytesMut::with_capacity(1024);
    buf.extend_from_slice(b"Hello World");
    buf.put(&b"googbye word!"[..]);
    buf.put_i32(42);
    buf.put_i64_le(0xdeadbeef);
    println!("{:?}", buf);
    let a = buf.split();
    let b = a.freeze();
    println!("{:?}", b);
    // println!("{:?}", a);
    println!("{:?}", buf);
    Ok(())
}
