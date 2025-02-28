use anyhow::{anyhow, Result};
use std::ops::Deref;
use std::ops::{Add, AddAssign, Mul};

pub struct Vector<T> {
    pub data: Vec<T>,
}

impl<T> Deref for Vector<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> Vector<T> {
    pub fn new(data: impl Into<Vec<T>>) -> Self {
        Self { data: data.into() }
    }
}

// pretend this is a heavy operation
pub fn dot_product<T>(mxa: Vector<T>, mxb: Vector<T>) -> Result<T>
where
    T: Copy + Default + Add<Output = T> + Mul<Output = T> + AddAssign,
{
    if mxa.len() != mxb.len() {
        return Err(anyhow!(
            "第一个矩阵的列数（column）和第二个矩阵的行数（row）不相同！"
        ));
    }

    let mut sum = T::default();
    for i in 0..mxa.len() {
        sum += mxa[i] * mxb[i];
    }

    Ok(sum)
}
