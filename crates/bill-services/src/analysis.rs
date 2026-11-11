//! 统计接口：年度分析。
//!
//! 行为基准为 Web 版 `api/analysis.py::yearly_analysis`。

use std::collections::{BTreeMap, BTreeSet};

use bill_core::{Dataset, Transaction};
use chrono::Datelike;
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
    rows.iter().map(|row| row.amount).sum()
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
