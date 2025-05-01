use anyhow::Result;
use derive_more::{Add, Debug, Display, From, Into};

#[derive(PartialEq, From, Into, Add, Debug)]
struct MyInt(i32);

#[derive(PartialEq, From, Into)]
struct Point2D {
    x: i32,
    y: i32,
}

#[derive(PartialEq, From, Add, Display, Debug)]
enum MyEnum {
    #[display("int: {_0}")]
    Int32(i32),
    UInt(u32),
    #[display("nothing")]
    Nothing,
}

fn main() -> Result<()> {
    let my_int = MyInt::from(27);
    let v = my_int + 20.into();
    let e1: MyEnum = 10i32.into();
    let e2: MyEnum = 27u32.into();
    let e3: MyEnum = MyEnum::Nothing;

    println!("{:?} {:?}, {:?}, {:?}", v, e1, e2, e3);
    Ok(())
}
