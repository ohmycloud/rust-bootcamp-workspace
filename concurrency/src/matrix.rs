use crate::vector::{dot_product, Vector};
use anyhow::{anyhow, Result};
use std::fmt::{Debug, Formatter};
use std::ops::{Add, AddAssign, Mul};
use std::sync::mpsc;
use std::{fmt, thread};

const NUM_THREADS: usize = 4;
pub struct MsgInput<T> {
    idx: usize,
    row: Vector<T>,
    col: Vector<T>,
}

pub struct MsgOutput<T> {
    idx: usize,
    value: T,
}

pub struct Msg<T> {
    pub input: MsgInput<T>,
    // sender to send the result
    pub sender: oneshot::Sender<MsgOutput<T>>,
}

impl<T> MsgInput<T> {
    pub fn new(idx: usize, row: Vector<T>, col: Vector<T>) -> Self {
        Self { idx, row, col }
    }
}

impl<T> Msg<T> {
    pub fn new(input: MsgInput<T>, sender: oneshot::Sender<MsgOutput<T>>) -> Self {
        Self { input, sender }
    }
}

impl<T> Mul for Matrix<T>
where
    T: Debug + Add<Output = T> + Mul<Output = T> + AddAssign + Copy + Default + Send + 'static,
{
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        matrix_multiply(&self, &rhs)
            .expect("第一个矩阵的列数（column）和第二个矩阵的行数（row）不相同！")
    }
}

pub struct Matrix<T> {
    pub data: Vec<T>,
    pub row: usize,
    pub col: usize,
}

pub fn matrix_multiply<T>(mxa: &Matrix<T>, mxb: &Matrix<T>) -> Result<Matrix<T>>
where
    T: Debug + Add<Output = T> + Mul<Output = T> + AddAssign + Copy + Default + Send + 'static,
{
    if mxa.col != mxb.row {
        return Err(anyhow!(
            "第一个矩阵的列数（column）和第二个矩阵的行数（row）不相同！"
        ));
    }

    // generate 4 threads which receive msg and do dot product
    let senders = (0..NUM_THREADS)
        .map(|_| {
            let (tx, rx) = mpsc::channel::<Msg<T>>();
            thread::spawn(move || {
                for msg in rx {
                    let value = dot_product(msg.input.row, msg.input.col)?;
                    if let Err(e) = msg.sender.send(MsgOutput {
                        idx: msg.input.idx,
                        value,
                    }) {
                        eprintln!("Send error: {:?}", e);
                    }
                }
                Ok::<_, anyhow::Error>(())
            });
            tx
        })
        .collect::<Vec<_>>();

    let matrix_len = mxa.row * mxb.col;
    let mut data = vec![T::default(); matrix_len];
    let mut receivers = Vec::with_capacity(matrix_len);
    for i in 0..mxa.row {
        for j in 0..mxb.col {
            let row = Vector::new(&mxa.data[i * mxa.col..(i + 1) * mxa.col]);
            let col_data = mxb.data[j..]
                .iter()
                .step_by(mxb.col)
                .copied()
                .collect::<Vec<_>>();
            let idx = i * mxb.col + j;
            let col = Vector::new(col_data);
            let input = MsgInput::new(i * mxb.col + j, row, col);
            let (tx, rx) = oneshot::channel();
            let msg = Msg::new(input, tx);

            if let Err(e) = senders[idx % NUM_THREADS].send(msg) {
                eprintln!("Send error {:?}", e);
            }
            receivers.push(rx);
        }
    }

    for rx in receivers {
        let output = rx.recv()?;
        data[output.idx] = output.value;
    }

    Ok(Matrix {
        data,
        row: mxa.row,
        col: mxb.col,
    })
}

impl<T> Matrix<T> {
    pub fn new(data: impl Into<Vec<T>>, row: usize, col: usize) -> Self {
        Self {
            data: data.into(),
            row,
            col,
        }
    }
}

impl<T: Debug> fmt::Display for Matrix<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{{")?;

        for i in 0..self.row {
            for j in 0..self.col {
                write!(f, "{:?}", self.data[i * self.col + j])?;
                if j != self.col - 1 {
                    write!(f, " ")?;
                }
            }

            if i != self.row - 1 {
                write!(f, ", ")?;
            }
        }
        write!(f, "}}")?;
        Ok(())
    }
}

impl<T: Debug> Debug for Matrix<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Matrix(row={}, col={}, {})", self.row, self.col, self)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matrix_multiply() -> Result<()> {
        let mxa = Matrix::new(vec![1, 2, 3, 4, 5, 6], 2, 3);
        let mxb = Matrix::new(vec![1, 2, 3, 4, 5, 6], 3, 2);
        let mxc = mxa * mxb;
        assert_eq!(mxc.col, 2);
        assert_eq!(mxc.row, 2);
        assert_eq!(format!("{:?}", mxc), "Matrix(row=2, col=2, {22 28, 49 64})");
        Ok(())
    }

    #[test]
    #[should_panic]
    fn test_matrix_can_not_multiply() {
        let mxa = Matrix::new(vec![1, 2, 3, 4, 5, 6], 2, 3);
        let mxb = Matrix::new(vec![1, 2, 3, 4], 2, 2);
        let _mxc = mxa * mxb;
    }
}
