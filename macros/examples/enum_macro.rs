use macros::EnumFrom;

#[allow(unused)]
#[derive(Debug, EnumFrom)]
enum Direction<T> {
    Up(Up<T>),
    Down,
    Left(u32),
    Right { z: u32 },
}

#[allow(unused)]
#[derive(Debug)]
struct Up<T> {
    speed: T,
}

impl<T> Up<T> {
    fn new(speed: T) -> Self {
        Self { speed }
    }
}

fn main() {
    let up: Direction<i32> = Up::new(42).into();
    let left: Direction<i32> = 10.into();
    println!("{:?} {:?}", up, left);
}
