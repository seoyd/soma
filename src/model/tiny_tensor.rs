use serde::{Deserialize, Serialize};

pub const TINY_TENSOR_MAX_ELEMENTS: usize = 4_096;

fn default_paper_only_true() -> bool {
    true
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TinyTensor1D {
    pub values: Vec<f32>,
    pub dim: usize,
    #[serde(default = "default_paper_only_true")]
    pub paper_only: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TinyTensor2D {
    pub values: Vec<f32>,
    pub rows: usize,
    pub cols: usize,
    #[serde(default = "default_paper_only_true")]
    pub paper_only: bool,
}

impl TinyTensor1D {
    pub fn is_finite(&self) -> bool {
        self.values.iter().all(|value| value.is_finite())
    }
}

impl TinyTensor2D {
    pub fn is_finite(&self) -> bool {
        self.values.iter().all(|value| value.is_finite())
    }
}

pub(crate) fn tiny_tensor_memory_ok(element_count: usize) -> bool {
    element_count <= TINY_TENSOR_MAX_ELEMENTS
}

pub fn zeros_1d(dim: usize) -> TinyTensor1D {
    TinyTensor1D {
        values: vec![0.0; dim],
        dim,
        paper_only: true,
    }
}

pub fn zeros_2d(rows: usize, cols: usize) -> TinyTensor2D {
    TinyTensor2D {
        values: vec![0.0; rows.saturating_mul(cols)],
        rows,
        cols,
        paper_only: true,
    }
}

pub fn from_vec_1d(values: Vec<f32>) -> Result<TinyTensor1D, String> {
    if values.iter().any(|value| !value.is_finite()) {
        return Err("tiny tensor 1d rejects NaN/Inf".to_string());
    }
    if !tiny_tensor_memory_ok(values.len()) {
        return Err("tiny tensor 1d exceeds small-memory limit".to_string());
    }
    Ok(TinyTensor1D {
        dim: values.len(),
        values,
        paper_only: true,
    })
}

pub fn from_vec_2d(rows: usize, cols: usize, values: Vec<f32>) -> Result<TinyTensor2D, String> {
    if rows == 0 || cols == 0 {
        return Err("tiny tensor 2d dimensions must be positive".to_string());
    }
    if rows.saturating_mul(cols) != values.len() {
        return Err("tiny tensor 2d row/col dimensions do not match value count".to_string());
    }
    if values.iter().any(|value| !value.is_finite()) {
        return Err("tiny tensor 2d rejects NaN/Inf".to_string());
    }
    if !tiny_tensor_memory_ok(values.len()) {
        return Err("tiny tensor 2d exceeds small-memory limit".to_string());
    }
    Ok(TinyTensor2D {
        values,
        rows,
        cols,
        paper_only: true,
    })
}

pub fn clamp_finite(x: f32) -> f32 {
    if x.is_finite() {
        x.clamp(-1_024.0, 1_024.0)
    } else {
        0.0
    }
}

pub fn tanh_approx(x: f32) -> f32 {
    let x = clamp_finite(x).clamp(-5.0, 5.0);
    let x2 = x * x;
    clamp_finite((x * (27.0 + x2)) / (27.0 + 9.0 * x2))
}

pub fn sigmoid_approx(x: f32) -> f32 {
    clamp_finite((tanh_approx(x * 0.5) + 1.0) * 0.5)
}

pub fn matvec(matrix: &TinyTensor2D, vector: &TinyTensor1D) -> Result<TinyTensor1D, String> {
    if matrix.cols != vector.dim {
        return Err("tiny tensor matvec dimension mismatch".to_string());
    }
    let mut output = Vec::with_capacity(matrix.rows);
    for row_index in 0..matrix.rows {
        let mut sum = 0.0_f32;
        let row_offset = row_index * matrix.cols;
        for col_index in 0..matrix.cols {
            sum += matrix.values[row_offset + col_index] * vector.values[col_index];
        }
        output.push(clamp_finite(sum));
    }
    from_vec_1d(output)
}

pub fn elem_add(left: &TinyTensor1D, right: &TinyTensor1D) -> Result<TinyTensor1D, String> {
    if left.dim != right.dim {
        return Err("tiny tensor elem_add dimension mismatch".to_string());
    }
    from_vec_1d(
        left.values
            .iter()
            .zip(&right.values)
            .map(|(left, right)| clamp_finite(left + right))
            .collect(),
    )
}

pub fn elem_mul(left: &TinyTensor1D, right: &TinyTensor1D) -> Result<TinyTensor1D, String> {
    if left.dim != right.dim {
        return Err("tiny tensor elem_mul dimension mismatch".to_string());
    }
    from_vec_1d(
        left.values
            .iter()
            .zip(&right.values)
            .map(|(left, right)| clamp_finite(left * right))
            .collect(),
    )
}

pub(crate) fn deterministic_tiny_value(seed: u64, index: usize, salt: u64) -> f32 {
    let mixed = seed
        .wrapping_mul(1_103_515_245)
        .wrapping_add((index as u64 + 1) * 12_345)
        .wrapping_add(salt * 97);
    let centered = (mixed % 97) as f32 - 48.0;
    clamp_finite(centered / 192.0)
}

pub(crate) fn deterministic_tiny_matrix(
    rows: usize,
    cols: usize,
    seed: u64,
    salt: u64,
) -> Result<TinyTensor2D, String> {
    let mut values = Vec::with_capacity(rows.saturating_mul(cols));
    for index in 0..rows.saturating_mul(cols) {
        values.push(deterministic_tiny_value(seed, index, salt));
    }
    from_vec_2d(rows, cols, values)
}
