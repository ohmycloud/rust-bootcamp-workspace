use anyhow::Result;

// In both the matcher and transcriber, repetitions are indicated by placing the tokens to be
// repeated inside $(…), followed by a repetition operator,
// optionally with a separator token between.
// The separator token can be any token other than a delimiter or one of the repetition operators,
// but ; and , are the most common. For instance, $( $i:ident ),* represents any number of
// identifiers separated by commas. Nested repetitions are permitted.
//
// The repetition operators are:
//
// * — indicates any number of repetitions.
// + — indicates any number but at least one.
// ? — indicates an optional fragment with zero or one occurrence.
// Since ? represents at most one occurrence, it cannot be used with a separator.
#[macro_export]
macro_rules! my_vec {
    // 匹配空的 Vec: let v = my_vec![];
    () => { Vec::new() };
    // 匹配(元素; 元素个数): let v = my_vec![42; 5];
    ($elem:expr; $n:expr) => {
        std::vec::from_elem($elem, $n)
    };
    // 匹配一个或多个表达式或者逗号
    ($($x:expr),+) => {
        {
            let mut temp_vec = Vec::new();
            $(
                temp_vec.push($x);
            )+
            temp_vec
        }
    };
    // 匹配末尾可能的逗号
    ($($x:expr),+ $(,)?) => {
        {
            <[_]>::into_vec(Box::new([$($x),*]))
        }
    };
}

fn main() -> Result<()> {
    let v: Vec<i32> = my_vec!["193".parse()?, 21, 33, "42".parse()?, 57,];
    println!("{:?}", v);

    // 创建 Vec 的另外一种形式
    let v1 = <[_]>::into_vec(Box::new([1, 2, 3, 4]));
    println!("{:?}", v1);
    Ok(())
}
