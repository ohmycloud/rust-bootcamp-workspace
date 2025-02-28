use macros::EnumFromDarling;

#[allow(unused)]
#[derive(Debug, EnumFromDarling)]
enum Direction {
    Up(Up),
    Down,
    Left(u32),
    Right { z: u32 },
}

#[allow(unused)]
#[derive(Debug)]
struct Up {
    speed: i32,
}

impl Up {
    fn new(speed: i32) -> Self {
        Self { speed }
    }
}

fn main() {
    let up: Direction = Up::new(42).into();
    let left: Direction = 10.into();
    println!("{:?}, {:?}", up, left);
}
