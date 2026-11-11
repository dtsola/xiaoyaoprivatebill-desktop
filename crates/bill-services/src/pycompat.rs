//! pandas / numpy 数值兼容层。
//!
//! 两处语义必须区分：
//! - `pandas_round2`：`DataFrame.round` / `Series.round`，缩放后按“四舍六入五成双”。
//! - `round2` 系列：Python 内置 `round()`，二进制值上四舍五入。
//!
//! 求和按 numpy 的成对求和实现，否则 `.875` 这类半分位会舍入到反方向。

/// numpy 成对求和：n<8 顺序累加；n<=128 用 8 个累加器；更大则二分。
pub fn numpy_sum(values: &[f64]) -> f64 {
    let n = values.len();
    if n < 8 {
        let mut result = 0.0;
        for value in values {
            result += value;
        }
        return result;
    }
    if n <= 128 {
        let mut accumulators = [0.0f64; 8];
        accumulators.copy_from_slice(&values[..8]);
        let mut index = 8;
        while index + 8 <= n {
            for (slot, value) in accumulators.iter_mut().zip(&values[index..index + 8]) {
                *slot += value;
            }
            index += 8;
        }
        let mut result = ((accumulators[0] + accumulators[1])
            + (accumulators[2] + accumulators[3]))
            + ((accumulators[4] + accumulators[5]) + (accumulators[6] + accumulators[7]));
        while index < n {
            result += values[index];
            index += 1;
        }
        return result;
    }
    let n2 = (n / 2) & !7;
    numpy_sum(&values[..n2]) + numpy_sum(&values[n2..])
}

/// pandas 的 `.round(2)`。
pub fn pandas_round2(value: f64) -> f64 {
    (value * 100.0).round_ties_even() / 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numpy_sum_matches_pairwise_accumulation() {
        let values: Vec<f64> = vec![
            752.8, 700.0, 600.0, 500.0, 400.0, 300.0, 200.0, 100.0, 50.0, 3.95,
        ];
        assert_eq!(numpy_sum(&values), 3606.75);
    }

    #[test]
    fn pandas_round_half_to_even() {
        assert_eq!(pandas_round2(752.875), 752.88);
        assert_eq!(pandas_round2(69.105), 69.1);
    }
}
