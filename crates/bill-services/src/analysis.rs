//! 统计接口：年度分析。
//!
//! 行为基准为 Web 版 `api/analysis.py::yearly_analysis`。

use std::collections::{BTreeMap, BTreeSet};

use bill_core::{Dataset, Transaction};
use chrono::{Datelike, Timelike};
use serde_json::{json, Value};

/// 年度分析查询参数。
#[derive(Debug, Default, Clone)]
pub struct YearlyQuery {
    /// 年份，缺省表示全部数据。
    pub year: Option<i32>,
    /// 金额下限（含），0 视为未设置（与 Python 的真值判断一致）。
    pub min_amount: Option<f64>,
    /// 金额上限（不含）。
    pub max_amount: Option<f64>,
}

/// 计算年度分析结果。
pub fn yearly_analysis(dataset: &Dataset, query: &YearlyQuery) -> Value {
    let current: Vec<&Transaction> = dataset
        .transactions
        .iter()
        .filter(|row| query.year.is_none_or(|year| row.time.year() == year))
        .collect();

    let last_year = query.year.map(|year| year - 1).or_else(|| {
        dataset
            .transactions
            .iter()
            .map(|row| row.time.year())
            .max()
            .map(|year| year - 1)
    });
    let last: Vec<&Transaction> = last_year
        .map(|year| {
            dataset
                .transactions
                .iter()
                .filter(|row| row.time.year() == year)
                .collect()
        })
        .unwrap_or_default();

    let current = apply_amount_filters(current, query);
    let last = apply_amount_filters(last, query);

    let current_expense = split(&current, "支出");
    let current_income = split(&current, "收入");
    let last_expense = split(&last, "支出");
    let last_income = split(&last, "收入");

    let total_expense = sum(&current_expense);
    let total_income = sum(&current_income);
    let balance = total_income - total_expense;
    let last_expense_sum = sum(&last_expense);
    let last_income_sum = sum(&last_income);
    let last_balance = last_income_sum - last_expense_sum;

    let active_days = current
        .iter()
        .map(|row| row.date.clone())
        .collect::<BTreeSet<_>>()
        .len();

    let prefix = query
        .year
        .map(|year| year.to_string())
        .unwrap_or_else(|| "None".to_owned());
    let months: Vec<String> = (1..=12)
        .map(|month| format!("{prefix}-{month:02}"))
        .collect();

    let monthly_expenses = monthly_sums(&current_expense, &months);
    let monthly_incomes = monthly_sums(&current_income, &months);

    let expense_categories = category_sums(&current_expense);
    let income_categories = category_sums(&current_income);

    let has_last = !last.is_empty();
    let stats = json!({
        "balance": balance,
        "total_expense": total_expense,
        "total_income": total_income,
        "expense_count": current_expense.len(),
        "income_count": current_income.len(),
        "total_count": current_expense.len() + current_income.len(),
        "active_days": active_days,
        "avg_transaction": if current_expense.is_empty() { 0.0 } else { total_expense / current_expense.len() as f64 },
        "avg_daily_expense": total_expense / active_days.max(1) as f64,
        "avg_monthly_income": total_income / 12.0,
        "expense_ratio": if total_income > 0.0 { total_expense / total_income * 100.0 } else { 0.0 },
        "comparisons": {
            "balance": comparison(balance - last_balance, last_balance, has_last),
            "expense": comparison(total_expense - last_expense_sum, last_expense_sum, has_last),
            "income": comparison(total_income - last_income_sum, last_income_sum, has_last),
            "count": count_comparison(
                current_expense.len() + current_income.len(),
                last_expense.len() + last_income.len(),
                has_last,
            ),
        }
    });

    json!({
        "trends": {
            "months": months,
            "expenses": monthly_expenses,
            "incomes": monthly_incomes,
        },
        "categories": {
            "expense": { "names": names(&expense_categories), "amounts": amounts(&expense_categories) },
            "income": { "names": names(&income_categories), "amounts": amounts(&income_categories) },
        },
        "categories_source": {
            "expense": source_records(&current_expense),
            "income": source_records(&current_income),
        },
        "yearly_stats": stats,
    })
}

fn apply_amount_filters<'a>(
    rows: Vec<&'a Transaction>,
    query: &YearlyQuery,
) -> Vec<&'a Transaction> {
    let min = query.min_amount.filter(|value| *value != 0.0);
    let max = query.max_amount;
    rows.into_iter()
        .filter(|row| min.is_none_or(|min| row.amount >= min))
        .filter(|row| max.is_none_or(|max| row.amount < max))
        .collect()
}

fn split<'a>(rows: &[&'a Transaction], direction: &str) -> Vec<&'a Transaction> {
    rows.iter()
        .copied()
        .filter(|row| row.direction == direction && !row.is_refund)
        .collect()
}

fn sum(rows: &[&Transaction]) -> f64 {
    // 显式以 +0.0 起算：Rust 的 f64 `Sum` 用 -0.0 作空集初值。
    rows.iter().fold(0.0, |acc, row| acc + row.amount)
}

fn monthly_sums(rows: &[&Transaction], months: &[String]) -> Vec<f64> {
    let keyed: BTreeMap<&str, f64> = rows.iter().fold(BTreeMap::new(), |mut acc, row| {
        *acc.entry(row.month.as_str()).or_insert(0.0) += row.amount;
        acc
    });
    months
        .iter()
        .map(|month| *keyed.get(month.as_str()).unwrap_or(&0.0))
        .collect()
}

fn category_sums(rows: &[&Transaction]) -> BTreeMap<String, f64> {
    rows.iter().fold(BTreeMap::new(), |mut acc, row| {
        *acc.entry(row.category.clone()).or_insert(0.0) += row.amount;
        acc
    })
}

fn names(map: &BTreeMap<String, f64>) -> Vec<String> {
    map.keys().cloned().collect()
}

fn amounts(map: &BTreeMap<String, f64>) -> Vec<f64> {
    map.values().copied().collect()
}

fn source_records(rows: &[&Transaction]) -> Vec<Value> {
    let mut grouped: BTreeMap<(String, String), f64> = BTreeMap::new();
    for row in rows {
        *grouped
            .entry((row.source.clone(), row.category.clone()))
            .or_insert(0.0) += row.amount;
    }
    grouped
        .into_iter()
        .map(|((source, category), amount)| {
            json!({ "来源": source, "交易分类": category, "金额": amount })
        })
        .collect()
}

fn comparison(change: f64, base: f64, has_last: bool) -> Value {
    if !has_last {
        return json!({ "change": Value::Null, "rate": Value::Null });
    }
    let rate = if base != 0.0 {
        json!((change / base.abs()) * 100.0)
    } else {
        json!(Value::Null)
    };
    json!({ "change": change, "rate": rate })
}

fn count_comparison(current: usize, last: usize, has_last: bool) -> Value {
    if !has_last {
        return json!({ "change": Value::Null, "rate": Value::Null });
    }
    let change = current as i64 - last as i64;
    let rate = if last != 0 {
        json!(change as f64 / last as f64 * 100.0)
    } else {
        json!(Value::Null)
    };
    json!({ "change": change, "rate": rate })
}

/// 月度分析查询参数。
#[derive(Debug, Clone, Copy)]
pub struct MonthlyQuery {
    /// 年份。
    pub year: i32,
    /// 月份 1–12。
    pub month: u32,
    /// 金额下限（含），0 视为未设置。
    pub min_amount: Option<f64>,
    /// 金额上限（不含）。
    pub max_amount: Option<f64>,
}

/// 计算月度分析结果。
pub fn monthly_analysis(dataset: &Dataset, query: &MonthlyQuery) -> Value {
    let current: Vec<&Transaction> = dataset
        .transactions
        .iter()
        .filter(|row| row.time.year() == query.year && row.time.month() == query.month)
        .collect();
    let (last_year, last_month) = if query.month > 1 {
        (query.year, query.month - 1)
    } else {
        (query.year - 1, 12)
    };
    let last: Vec<&Transaction> = dataset
        .transactions
        .iter()
        .filter(|row| row.time.year() == last_year && row.time.month() == last_month)
        .collect();

    let filters = YearlyQuery {
        year: None,
        min_amount: query.min_amount,
        max_amount: query.max_amount,
    };
    let current = apply_amount_filters(current, &filters);
    let last = apply_amount_filters(last, &filters);

    let current_expense = split(&current, "支出");
    let current_income = split(&current, "收入");
    let total_expense = sum(&current_expense);
    let total_income = sum(&current_income);
    let balance = total_income - total_expense;
    let last_expense = sum(&split(&last, "支出"));
    let last_income = sum(&split(&last, "收入"));
    let last_balance = last_income - last_expense;

    let days = days_in_month(query.year, query.month);
    let dates: Vec<String> = (1..=days)
        .map(|day| format!("{}-{:02}-{day:02}", query.year, query.month))
        .collect();
    let daily_expenses = daily_sums(&current_expense, &dates);
    let daily_incomes = daily_sums(&current_income, &dates);

    let expense_categories = category_sums(&current_expense);
    let income_categories = category_sums(&current_income);

    json!({
        "stats": {
            "balance": balance,
            "total_expense": total_expense,
            "total_income": total_income,
            "expense_count": current_expense.len(),
            "income_count": current_income.len(),
            "comparisons": {
                "balance": month_comparison(balance - last_balance, last_balance),
                "expense": month_comparison(total_expense - last_expense, last_expense),
                "income": month_comparison(total_income - last_income, last_income),
            }
        },
        "daily_data": {
            "expense": { "dates": dates, "amounts": daily_expenses },
            "income": { "dates": dates, "amounts": daily_incomes },
        },
        "categories": {
            "expense": { "names": names(&expense_categories), "amounts": amounts(&expense_categories) },
            "income": { "names": names(&income_categories), "amounts": amounts(&income_categories) },
        },
        "categories_source": {
            "expense": source_records(&current_expense),
            "income": source_records(&current_income),
        }
    })
}

fn daily_sums(rows: &[&Transaction], dates: &[String]) -> Vec<f64> {
    let keyed: BTreeMap<&str, f64> = rows.iter().fold(BTreeMap::new(), |mut acc, row| {
        *acc.entry(row.date.as_str()).or_insert(0.0) += row.amount;
        acc
    });
    dates
        .iter()
        .map(|date| *keyed.get(date.as_str()).unwrap_or(&0.0))
        .collect()
}

fn days_in_month(year: i32, month: u32) -> u32 {
    let (next_year, next_month) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };
    let first_next =
        chrono::NaiveDate::from_ymd_opt(next_year, next_month, 1).expect("下月一日合法");
    let first = chrono::NaiveDate::from_ymd_opt(year, month, 1).expect("本月一日合法");
    (first_next - first).num_days() as u32
}

fn month_comparison(change: f64, base: f64) -> Value {
    let rate = if base != 0.0 {
        json!((change / base.abs()) * 100.0)
    } else {
        json!(Value::Null)
    };
    json!({ "change": change, "rate": rate })
}

/// 时间分析：24 小时分布 + 分类的工作日/周末分布。
pub fn time_analysis(dataset: &Dataset, year: Option<i32>, filter_type: &str) -> Value {
    use chrono::Weekday;

    let rows: Vec<&Transaction> = dataset
        .transactions
        .iter()
        .filter(|row| row.direction == "支出" && !row.is_refund && row.amount > 0.0)
        .filter(|row| year.is_none_or(|value| row.time.year() == value))
        .filter(|row| match filter_type {
            "large" => row.amount > 1000.0,
            "small" => row.amount <= 1000.0,
            _ => true,
        })
        .collect();

    let mut hour_sums = [0.0f64; 24];
    let mut hour_counts = [0usize; 24];
    let mut grouped: BTreeMap<String, (f64, usize, f64, usize)> = BTreeMap::new();
    for row in &rows {
        let hour = row.time.hour() as usize;
        hour_sums[hour] += row.amount;
        hour_counts[hour] += 1;

        let entry = grouped
            .entry(row.category.clone())
            .or_insert((0.0, 0, 0.0, 0));
        let weekend = matches!(row.time.weekday(), Weekday::Sat | Weekday::Sun);
        if weekend {
            entry.2 += row.amount;
            entry.3 += 1;
        } else {
            entry.0 += row.amount;
            entry.1 += 1;
        }
    }

    let amounts: Vec<f64> = hour_sums.iter().map(|value| round2(*value)).collect();

    let mut items: Vec<(String, (f64, usize, f64, usize))> = grouped
        .into_iter()
        .filter(|(_, (weekday, _, weekend, _))| weekday + weekend > 0.0)
        .collect();
    items.sort_by(|left, right| {
        let left_total = left.1 .0 + left.1 .2;
        let right_total = right.1 .0 + right.1 .2;
        right_total
            .total_cmp(&left_total)
            .then(left.0.cmp(&right.0))
    });

    let mut weekday_weekend = serde_json::Map::new();
    for (category, (weekday_amount, weekday_count, weekend_amount, weekend_count)) in items {
        let total = weekday_amount + weekend_amount;
        weekday_weekend.insert(
            category,
            json!({
                "weekday": {
                    "amount": weekday_amount,
                    "count": weekday_count,
                    "percentage": round1(weekday_amount / total * 100.0),
                },
                "weekend": {
                    "amount": weekend_amount,
                    "count": weekend_count,
                    "percentage": round1(weekend_amount / total * 100.0),
                }
            }),
        );
    }

    json!({
        "hourly": { "amounts": amounts, "counts": hour_counts },
        "weekday_weekend": weekday_weekend,
    })
}

fn round1(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}

fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

/// 热力图数据：每日收支与分位数。
pub fn daily_data(dataset: &Dataset, year: Option<i32>, filter_type: &str) -> Value {
    let rows: Vec<&Transaction> = dataset
        .transactions
        .iter()
        .filter(|row| match filter_type {
            "large" => row.amount > 1000.0,
            "small" => row.amount <= 1000.0,
            _ => true,
        })
        .filter(|row| year.is_none_or(|value| row.time.year() == value))
        .filter(|row| row.direction == "收入" || row.direction == "支出")
        .collect();

    let mut grouped: BTreeMap<(String, String), (f64, usize)> = BTreeMap::new();
    for row in &rows {
        let entry = grouped
            .entry((row.date.clone(), row.direction.clone()))
            .or_insert((0.0, 0));
        entry.0 += row.amount;
        entry.1 += 1;
    }

    let dates: Vec<String> = {
        let mut values: Vec<String> = grouped.keys().map(|(date, _)| date.clone()).collect();
        values.dedup();
        values
    };

    let mut expense = Vec::new();
    let mut income = Vec::new();
    let mut transaction = Vec::new();
    for date in &dates {
        let mut count = 0usize;
        if let Some((amount, rows)) = grouped.get(&(date.clone(), "支出".into())) {
            expense.push(json!([date, amount]));
            count += rows;
        }
        if let Some((amount, rows)) = grouped.get(&(date.clone(), "收入".into())) {
            income.push(json!([date, amount]));
            count += rows;
        }
        transaction.push(json!([date, count]));
    }

    let expense_amounts: Vec<f64> = expense
        .iter()
        .filter_map(|item| item.get(1).and_then(Value::as_f64))
        .collect();
    let income_amounts: Vec<f64> = income
        .iter()
        .filter_map(|item| item.get(1).and_then(Value::as_f64))
        .collect();

    json!({
        "expense": expense,
        "income": income,
        "transaction": transaction,
        "expense_quantiles": quantiles(&expense_amounts),
        "income_quantiles": quantiles(&income_amounts),
    })
}

/// numpy.quantile 的线性插值口径，取 0.2/0.4/0.6/0.8。
fn quantiles(values: &[f64]) -> Vec<f64> {
    if values.is_empty() {
        return Vec::new();
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(|left, right| left.total_cmp(right));
    [0.2, 0.4, 0.6, 0.8]
        .iter()
        .map(|q| {
            let position = q * (sorted.len() - 1) as f64;
            let lower = position.floor() as usize;
            let upper = position.ceil() as usize;
            let weight = position - lower as f64;
            let value = if lower == upper {
                sorted[lower]
            } else {
                sorted[lower] + (sorted[upper] - sorted[lower]) * weight
            };
            round2(value)
        })
        .collect()
}

/// 大额交易：仅支出，按金额降序取前 N 条（并列保持原顺序）。
pub fn top_transactions(dataset: &Dataset, limit: usize, min_amount: f64) -> Value {
    let mut rows: Vec<&Transaction> = dataset
        .transactions
        .iter()
        .filter(|row| row.direction == "支出" && row.amount >= min_amount)
        .collect();
    rows.sort_by(|left, right| right.amount.total_cmp(&left.amount));
    rows.truncate(limit);

    let items: Vec<Value> = rows
        .iter()
        .map(|row| {
            json!({
                "time": row.time.format("%Y-%m-%d %H:%M:%S").to_string(),
                "date": row.date,
                "category": row.category,
                "description": row.description,
                "amount": row.amount,
                "status": row.status,
                "counterparty": row.counterparty,
            })
        })
        .collect();

    json!({ "success": true, "transactions": items })
}

/// 汇总数据。`current_month` 由调用方提供（`YYYY-MM`），便于测试。
pub fn summary(dataset: &Dataset, current_month: &str) -> Value {
    let mut monthly: BTreeMap<String, f64> = BTreeMap::new();
    let mut total_expense = 0.0;
    let mut total_income = 0.0;
    let mut expense_count = 0usize;

    for row in &dataset.transactions {
        match row.direction.as_str() {
            "支出" => {
                total_expense += row.amount;
                expense_count += 1;
                *monthly.entry(row.month.clone()).or_insert(0.0) += row.amount;
            }
            "收入" => total_income += row.amount,
            _ => {}
        }
    }

    let Some((latest_month, latest_expense)) = monthly.iter().next_back() else {
        return json!({ "error": "没有可用的账单数据" });
    };

    let current_expense = monthly.get(current_month).copied();
    let display_month = if current_expense.is_some() {
        current_month
    } else {
        latest_month.as_str()
    };
    let display_expense = current_expense.unwrap_or(*latest_expense);

    // 复刻 D5：上月取「倒数第二个月」，而不是展示月的前一个月。
    let prev_expense = if monthly.len() > 1 {
        monthly
            .values()
            .nth(monthly.len() - 2)
            .copied()
            .unwrap_or(display_expense)
    } else {
        display_expense
    };
    let monthly_avg = monthly.values().sum::<f64>() / monthly.len() as f64;

    json!({
        "total_expense": round2(total_expense),
        "total_income": round2(total_income),
        "balance": round2(total_income - total_expense),
        "monthly_avg": round2(monthly_avg),
        "current_month_expense": round2(display_expense),
        "prev_monthly_avg": round2(prev_expense),
        "month_count": monthly.len(),
        "transaction_count": expense_count,
        "current_month": display_month,
        "has_current_month_data": current_expense.is_some(),
    })
}

/// 首页概览数据。
pub fn overview_data(dataset: &Dataset, year: Option<i32>, filter_type: &str) -> Value {
    let available_years = dataset.available_years();
    let Some(default_year) = available_years.first().copied() else {
        return json!({ "error": "没有可用的账单数据" });
    };
    let year = year.unwrap_or(default_year);

    let year_rows: Vec<&Transaction> = dataset
        .transactions
        .iter()
        .filter(|row| row.time.year() == year)
        .filter(|row| match filter_type {
            "large" => row.amount.abs() >= 1000.0,
            "small" => row.amount.abs() < 1000.0,
            _ => true,
        })
        .collect();

    let expense: Vec<&Transaction> = year_rows
        .iter()
        .copied()
        .filter(|row| row.direction == "支出" && !row.is_refund)
        .collect();
    let income: Vec<&Transaction> = year_rows
        .iter()
        .copied()
        .filter(|row| row.direction == "收入" && !row.is_refund)
        .collect();

    let total_expense: f64 = expense.iter().map(|row| row.amount).sum();
    let total_income: f64 = income.iter().map(|row| row.amount).sum();
    let active_days = year_rows
        .iter()
        .map(|row| row.date.clone())
        .collect::<BTreeSet<_>>()
        .len();

    let months: Vec<String> = (1..=12).map(|month| format!("{year}-{month:02}")).collect();
    let monthly: BTreeMap<&str, f64> = expense.iter().fold(BTreeMap::new(), |mut acc, row| {
        *acc.entry(row.month.as_str()).or_insert(0.0) += row.amount;
        acc
    });
    let amounts: Vec<f64> = months
        .iter()
        .map(|month| round2(*monthly.get(month.as_str()).unwrap_or(&0.0)))
        .collect();

    let mut categories: Vec<(String, f64)> = category_totals(&expense);
    categories.sort_by(|left, right| right.1.total_cmp(&left.1).then(left.0.cmp(&right.0)));

    json!({
        "available_years": available_years,
        // Web 版 `overview_data` 返回字符串年份。
        "current_year": year.to_string(),
        "yearly_stats": {
            "total_expense": round2(total_expense),
            "total_income": round2(total_income),
            "balance": round2(total_income - total_expense),
            "expense_count": expense.len(),
            "income_count": income.len(),
            "total_count": expense.len() + income.len(),
            "active_days": active_days,
            "avg_transaction": if expense.is_empty() { 0.0 } else { round2(total_expense / expense.len() as f64) },
            "avg_daily_expense": round2(total_expense / 365.0),
            "avg_monthly_income": round2(total_income / 12.0),
            "expense_ratio": if total_income > 0.0 { round2(total_expense / total_income * 100.0) } else { 0.0 },
        },
        "months": months,
        "amounts": amounts,
        "categories": categories.iter().map(|(name, _)| name.clone()).collect::<Vec<_>>(),
        "amounts_by_category": categories.iter().map(|(_, amount)| *amount).collect::<Vec<_>>(),
    })
}

fn category_totals(rows: &[&Transaction]) -> Vec<(String, f64)> {
    let mut grouped: BTreeMap<String, f64> = BTreeMap::new();
    for row in rows {
        *grouped.entry(row.category.clone()).or_insert(0.0) += row.amount;
    }
    grouped
        .into_iter()
        .map(|(name, total)| (name, round2(total)))
        .collect()
}

// ============ P2b：兼容接口（裸对象） ============

/// 基础统计，对应 Web 版 `calculate_monthly_stats`。
#[derive(Debug, Default, Clone, Copy)]
struct Stats {
    balance: f64,
    total_expense: f64,
    total_income: f64,
    expense_count: usize,
    income_count: usize,
    total_count: usize,
    active_days: usize,
    avg_transaction: f64,
    avg_daily_expense: f64,
    expense_ratio: f64,
    empty: bool,
}

impl Stats {
    /// 输出与 Python 一致的整数/浮点混用：空聚合在某些字段上是 `int 0`。
    fn as_json(&self) -> Value {
        if self.empty {
            return json!({
                "balance": 0,
                "total_expense": 0,
                "total_income": 0,
                "expense_count": 0,
                "income_count": 0,
                "total_count": 0,
                "active_days": 0,
                "avg_transaction": 0,
                "avg_daily_expense": 0,
                "expense_ratio": 0,
            });
        }
        json!({
            "balance": self.balance,
            "total_expense": if self.expense_count == 0 { json!(0) } else { json!(self.total_expense) },
            "total_income": if self.income_count == 0 { json!(0) } else { json!(self.total_income) },
            "expense_count": self.expense_count,
            "income_count": self.income_count,
            "total_count": self.total_count,
            "active_days": self.active_days,
            "avg_transaction": if self.expense_count == 0 { json!(0) } else { json!(self.avg_transaction) },
            "avg_daily_expense": self.avg_daily_expense,
            "expense_ratio": if self.total_income > 0.0 { json!(self.expense_ratio) } else { json!(0) },
        })
    }
}

fn calc_stats(rows: &[&Transaction]) -> Stats {
    if rows.is_empty() {
        return Stats {
            empty: true,
            ..Stats::default()
        };
    }

    let expense: Vec<&Transaction> = rows
        .iter()
        .copied()
        .filter(|row| row.direction == "支出" && !row.is_refund)
        .collect();
    let income: Vec<&Transaction> = rows
        .iter()
        .copied()
        .filter(|row| row.direction == "收入" && !row.is_refund)
        .collect();
    let total_expense = sum(&expense);
    let total_income = sum(&income);
    let active_days = rows
        .iter()
        .map(|row| row.date.as_str())
        .collect::<BTreeSet<_>>()
        .len();

    Stats {
        balance: total_income - total_expense,
        total_expense,
        total_income,
        expense_count: expense.len(),
        income_count: income.len(),
        total_count: expense.len() + income.len(),
        active_days,
        avg_transaction: if expense.is_empty() {
            0.0
        } else {
            round2(total_expense / expense.len() as f64)
        },
        avg_daily_expense: round2(total_expense / active_days.max(1) as f64),
        expense_ratio: if total_income > 0.0 {
            round2(total_expense / total_income * 100.0)
        } else {
            0.0
        },
        empty: false,
    }
}

fn year_rows<'a>(dataset: &'a Dataset, year: i32, filter_type: &str) -> Vec<&'a Transaction> {
    dataset
        .transactions
        .iter()
        .filter(|row| row.time.year() == year)
        .filter(|row| match filter_type {
            // 与 Web 版一致：large 为 >=1000，small 为 <1000。
            "large" => row.amount >= 1000.0,
            "small" => row.amount < 1000.0,
            _ => true,
        })
        .collect()
}

fn change_rate(current: f64, previous: f64) -> Value {
    if previous == 0.0 {
        if current == 0.0 {
            return json!(0);
        }
        return Value::Null;
    }
    if current == 0.0 {
        return json!(-100);
    }
    json!(round2((current - previous) / previous.abs() * 100.0))
}

/// 年度数据（兼容接口，裸对象）。
pub fn yearly_data(dataset: &Dataset, year: Option<i32>, filter_type: &str) -> Value {
    let available_years = dataset.available_years();
    let Some(default_year) = available_years.first().copied() else {
        return json!({ "error": "没有可用的账单数据" });
    };
    let year = year.unwrap_or(default_year);

    let current = year_rows(dataset, year, filter_type);
    let last = year_rows(dataset, year - 1, filter_type);
    let current_stats = calc_stats(&current);
    let mut stats = current_stats.as_json();

    let keys = [
        "balance_change",
        "expense_change",
        "income_change",
        "transaction_change",
        "balance_change_rate",
        "expense_change_rate",
        "income_change_rate",
        "transaction_change_rate",
    ];
    if let Some(object) = stats.as_object_mut() {
        if last.is_empty() {
            for key in keys {
                object.insert(key.to_owned(), Value::Null);
            }
        } else {
            let last_stats = calc_stats(&last);
            object.insert(
                "balance_change".into(),
                json!(current_stats.balance - last_stats.balance),
            );
            object.insert(
                "expense_change".into(),
                json!(current_stats.total_expense - last_stats.total_expense),
            );
            object.insert(
                "income_change".into(),
                json!(current_stats.total_income - last_stats.total_income),
            );
            object.insert(
                "transaction_change".into(),
                json!(current_stats.total_count as i64 - last_stats.total_count as i64),
            );
            object.insert(
                "balance_change_rate".into(),
                change_rate(current_stats.balance, last_stats.balance),
            );
            object.insert(
                "expense_change_rate".into(),
                change_rate(current_stats.total_expense, last_stats.total_expense),
            );
            object.insert(
                "income_change_rate".into(),
                change_rate(current_stats.total_income, last_stats.total_income),
            );
            object.insert(
                "transaction_change_rate".into(),
                change_rate(
                    current_stats.total_count as f64,
                    last_stats.total_count as f64,
                ),
            );
        }
    }

    let months: Vec<&str> = current
        .iter()
        .map(|row| row.month.as_str())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();

    let mut expenses = Vec::new();
    let mut incomes = Vec::new();
    for month in &months {
        let month_rows: Vec<&Transaction> = current
            .iter()
            .copied()
            .filter(|row| row.month == *month)
            .collect();
        let month_expense: Vec<&Transaction> = month_rows
            .iter()
            .copied()
            .filter(|row| row.direction == "支出" && !row.is_refund)
            .collect();
        let month_income: Vec<&Transaction> = month_rows
            .iter()
            .copied()
            .filter(|row| row.direction == "收入" && !row.is_refund)
            .collect();
        expenses.push(round2(sum(&month_expense)));
        incomes.push(round2(sum(&month_income)));
    }

    let expense_rows: Vec<&Transaction> = current
        .iter()
        .copied()
        .filter(|row| row.direction == "支出" && !row.is_refund)
        .collect();
    let mut categories = category_totals(&expense_rows);
    categories.sort_by(|left, right| right.1.total_cmp(&left.1).then(left.0.cmp(&right.0)));

    json!({
        "yearly_stats": stats,
        "months": months,
        "expenses": expenses,
        "incomes": incomes,
        "categories": categories.iter().map(|(name, _)| name.clone()).collect::<Vec<_>>(),
        "amounts_by_category": categories.iter().map(|(_, amount)| *amount).collect::<Vec<_>>(),
        "available_years": available_years,
        "current_year": year,
    })
}

fn parse_month(month: &str) -> (i32, u32) {
    let mut parts = month.split('-');
    let year = parts
        .next()
        .and_then(|value| value.parse().ok())
        .unwrap_or(0);
    let number = parts
        .next()
        .and_then(|value| value.parse().ok())
        .unwrap_or(1);
    (year, number)
}

/// 月度数据（兼容接口，带 `success` 外层）。
pub fn monthly_data(
    dataset: &Dataset,
    year: Option<i32>,
    month: Option<u32>,
    filter_type: &str,
) -> Value {
    let Some(latest) = dataset
        .transactions
        .iter()
        .map(|row| row.month.as_str())
        .max()
    else {
        return json!({ "success": false, "error": "没有可用的账单数据" });
    };
    let (default_year, default_month) = parse_month(latest);
    let year = year.unwrap_or(default_year);
    let month = month.unwrap_or(default_month);

    let current_month = format!("{year}-{month:02}");
    let (last_year, last_month) = if month > 1 {
        (year, month - 1)
    } else {
        (year - 1, 12)
    };
    let last_month = format!("{last_year}-{last_month:02}");

    let filter = |row: &&Transaction| match filter_type {
        "large" => row.amount >= 1000.0,
        "small" => row.amount < 1000.0,
        _ => true,
    };
    let current: Vec<&Transaction> = dataset
        .transactions
        .iter()
        .filter(|row| row.month == current_month)
        .filter(filter)
        .collect();
    let last: Vec<&Transaction> = dataset
        .transactions
        .iter()
        .filter(|row| row.month == last_month)
        .filter(filter)
        .collect();

    let current_stats = calc_stats(&current);
    let mut data = current_stats.as_json();

    if !last.is_empty() {
        let last_stats = calc_stats(&last);
        let comparison = |current: f64, previous: f64| {
            let diff = current - previous;
            // 上期为 0 时 Python 显式返回整数 0。
            let rate = if previous != 0.0 {
                json!(diff / previous.abs() * 100.0)
            } else {
                json!(0)
            };
            json!({ "diff": diff, "rate": rate })
        };
        if let Some(object) = data.as_object_mut() {
            object.insert(
                "comparisons".into(),
                json!({
                    "total_expense": comparison(current_stats.total_expense, last_stats.total_expense),
                    "total_income": comparison(current_stats.total_income, last_stats.total_income),
                    "balance": comparison(current_stats.balance, last_stats.balance),
                }),
            );
        }
    }

    json!({ "success": true, "data": data })
}

/// 过滤后的月度分析（兼容接口，裸对象）。
pub fn filtered_monthly_analysis(dataset: &Dataset, filter_type: &str) -> Value {
    let rows: Vec<&Transaction> = dataset
        .transactions
        .iter()
        .filter(|row| row.direction == "支出" && !row.is_refund && row.amount > 0.0)
        .filter(|row| match filter_type {
            // 注意：此处 large 为 >1000、small 为 <=1000，与年度/月度不同。
            "large" => row.amount > 1000.0,
            "small" => row.amount <= 1000.0,
            _ => true,
        })
        .collect();

    let months: Vec<&str> = rows
        .iter()
        .map(|row| row.month.as_str())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();

    let mut totals = Vec::new();
    let mut counts = Vec::new();
    let mut daily_averages = Vec::new();
    let mut mom_rates = Vec::new();
    let mut moving_averages = Vec::new();

    for (index, month) in months.iter().enumerate() {
        let month_rows: Vec<&Transaction> = rows
            .iter()
            .copied()
            .filter(|row| row.month == *month)
            .collect();
        let total = round2(sum(&month_rows));
        let active_days = month_rows
            .iter()
            .map(|row| row.date.as_str())
            .collect::<BTreeSet<_>>()
            .len();

        totals.push(total);
        counts.push(month_rows.len());
        daily_averages.push(round2(total / active_days.max(1) as f64));

        if index == 0 {
            mom_rates.push(0.0);
        } else {
            let previous = totals[index - 1];
            mom_rates.push(if previous != 0.0 {
                round2((total / previous - 1.0) * 100.0)
            } else {
                0.0
            });
        }

        let start = index.saturating_sub(2);
        let window = &totals[start..=index];
        let average = window.iter().sum::<f64>() / window.len() as f64;
        moving_averages.push(round2(average));
    }

    let categories: Vec<&str> = rows
        .iter()
        .map(|row| row.category.as_str())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();

    let category_expenses: Vec<Vec<f64>> = months
        .iter()
        .map(|month| {
            categories
                .iter()
                .map(|category| {
                    rows.iter()
                        .filter(|row| row.month == *month && row.category == *category)
                        .map(|row| row.amount)
                        .sum::<f64>()
                })
                .collect()
        })
        .collect();

    json!({
        "months": months,
        "total_expenses": totals,
        "transaction_counts": counts,
        "daily_averages": daily_averages,
        "mom_rates": mom_rates,
        "moving_averages": moving_averages,
        "categories": categories,
        "category_expenses": category_expenses,
    })
}
