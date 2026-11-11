//! P2-8 单元 3 契约测试：10 类图表数据。
//!
//! 期望值由 Web 版 `services/analysis.py::analyze_sankey` 与
//! `services/generators.py` 在同一份 24 行数据上运行后取得。

use bill_core::{Dataset, Transaction};
use bill_services::charts::{
    analyze_sankey, generate_boxplot_data, generate_chord_data, generate_funnel_data,
    generate_heatmap_data, generate_pareto_data, generate_quadrant_data, generate_radar_data,
    generate_themeriver_data, generate_wordcloud_data,
};
use serde_json::json;

const ROWS: [&str; 24] = [
    "2024-01-05|9|20.0|咖啡店|餐饮美食|支付宝",
    "2024-01-06|9|20.0|咖啡店|餐饮美食|支付宝",
    "2024-01-07|9|20.0|咖啡店|餐饮美食|支付宝",
    "2024-01-08|9|20.0|咖啡店|餐饮美食|支付宝",
    "2024-01-09|9|20.0|咖啡店|餐饮美食|支付宝",
    "2024-01-10|9|20.0|咖啡店|餐饮美食|支付宝",
    "2024-01-11|10|25.0|奶茶店|餐饮美食|微信",
    "2024-01-12|10|25.0|奶茶店|餐饮美食|微信",
    "2024-01-13|11|10.0|便利店|购物|支付宝",
    "2024-01-14|12|30.0|超市|购物|微信",
    "2024-02-10|23|100.0|深夜食堂|餐饮美食|支付宝",
    "2024-02-11|2|50.0|凌晨超市|购物|微信",
    "2024-02-12|22|30.0|便利店|购物|微信",
    "2024-02-13|5|40.0|早市|餐饮美食|支付宝",
    "2024-01-01|8|15.0|视频会员|娱乐|支付宝",
    "2024-02-01|8|15.0|视频会员|娱乐|支付宝",
    "2024-03-01|8|15.0|视频会员|娱乐|支付宝",
    "2024-01-05|8|10.0|云存储|娱乐|微信",
    "2024-02-05|8|10.0|云存储|娱乐|微信",
    "2024-03-05|8|10.0|云存储|娱乐|微信",
    "2024-01-10|8|200.0|健身房|娱乐|支付宝",
    "2024-02-10|8|200.0|健身房|娱乐|支付宝",
    "2024-04-15|14|500.0|家电店|购物|支付宝",
    "2024-05-20|15|300.0|服装店|购物|微信",
];

fn dataset() -> Dataset {
    Dataset {
        transactions: ROWS
            .iter()
            .map(|row| {
                let parts: Vec<&str> = row.split('|').collect();
                let time = chrono::NaiveDate::parse_from_str(parts[0], "%Y-%m-%d")
                    .expect("日期")
                    .and_hms_opt(parts[1].parse().expect("小时"), 0, 0)
                    .expect("时间");
                Transaction {
                    time,
                    month: time.format("%Y-%m").to_string(),
                    date: parts[0].to_string(),
                    direction: "支出".into(),
                    amount: parts[2].parse().expect("金额"),
                    category: parts[4].into(),
                    description: String::new(),
                    counterparty: parts[3].into(),
                    payment_method: String::new(),
                    status: "交易成功".into(),
                    is_refund: false,
                    source: parts[5].into(),
                }
            })
            .collect(),
        files: Vec::new(),
    }
}

fn empty() -> Dataset {
    Dataset {
        transactions: Vec::new(),
        files: Vec::new(),
    }
}

#[test]
fn sankey_nodes_follow_first_appearance_and_links_follow_groupby_order() {
    let data = analyze_sankey(&dataset());
    let nodes = data["nodes"].as_array().expect("nodes");
    assert_eq!(nodes.len(), 5);
    assert_eq!(nodes[0], json!({"name": "支付宝", "category": "source"}));
    assert_eq!(nodes[1], json!({"name": "微信", "category": "source"}));
    assert_eq!(
        nodes[2],
        json!({"name": "餐饮美食", "category": "category"})
    );

    let links = data["links"].as_array().expect("links");
    assert_eq!(links.len(), 6, "娱乐/购物/餐饮 各 2 条来源边");
    assert_eq!(links[0], json!({"source": 1, "target": 4, "value": 30.0}));
    assert_eq!(links[5], json!({"source": 0, "target": 2, "value": 260.0}));
}

#[test]
fn chord_has_seven_weekday_nodes_and_sorted_links() {
    let data = generate_chord_data(&dataset());
    let nodes = data["nodes"].as_array().expect("nodes");
    assert_eq!(nodes.len(), 10, "7 个星期 + 3 个分类");
    assert_eq!(nodes[0], json!({"name": "周一", "category": "weekday"}));
    assert_eq!(nodes[6], json!({"name": "周日", "category": "weekday"}));
    assert_eq!(nodes[7], json!({"name": "购物", "category": "category"}));

    let links = data["links"].as_array().expect("links");
    assert_eq!(links.len(), 16);
    assert_eq!(
        links[0],
        json!({"source": "周一", "target": "娱乐", "value": 25.0})
    );
    assert_eq!(
        links[15],
        json!({"source": "周日", "target": "餐饮美食", "value": 20.0})
    );
}

#[test]
fn funnel_keeps_empty_bins_and_sorts_by_value() {
    let data = generate_funnel_data(&dataset());
    assert_eq!(
        data,
        json!([
            {"name": "100-500元", "value": 800.0},
            {"name": "500-1000元", "value": 500.0},
            {"name": "0-50元", "value": 355.0},
            {"name": "50-100元", "value": 50.0},
            {"name": "1000-5000元", "value": 0.0},
            {"name": ">5000元", "value": 0.0},
        ])
    );
}

#[test]
fn quadrant_filters_small_merchants() {
    let data = generate_quadrant_data(&dataset());
    assert_eq!(
        data,
        json!([
            {"name": "健身房", "category": "娱乐", "frequency": 2, "avg_amount": 200.0, "total_amount": 400.0},
            {"name": "咖啡店", "category": "餐饮美食", "frequency": 6, "avg_amount": 20.0, "total_amount": 120.0},
            {"name": "奶茶店", "category": "餐饮美食", "frequency": 2, "avg_amount": 25.0, "total_amount": 50.0},
        ])
    );
}

#[test]
fn radar_uses_category_top_eight_and_quarter_numbers() {
    let data = generate_radar_data(&dataset());
    assert_eq!(
        data["indicator"],
        json!([
            {"name": "购物", "max": 880.0000000000001},
            {"name": "娱乐", "max": 880.0000000000001},
            {"name": "餐饮美食", "max": 880.0000000000001},
        ])
    );
    assert_eq!(
        data["series"],
        json!([
            {"name": "Q1", "value": [120.0, 475.0, 310.0]},
            {"name": "Q2", "value": [800.0, 0.0, 0.0]},
        ])
    );
}

#[test]
fn wordcloud_drops_small_merchants_and_keeps_top_hundred() {
    let data = generate_wordcloud_data(&dataset());
    let items = data.as_array().expect("数组");
    assert_eq!(items.len(), 12);
    assert_eq!(items[0], json!({"name": "家电店", "value": 500.0}));
    assert_eq!(items[1], json!({"name": "健身房", "value": 400.0}));
    assert_eq!(items[11], json!({"name": "超市", "value": 30.0}));
}

#[test]
fn themeriver_expands_every_month_category_pair() {
    let data = generate_themeriver_data(&dataset());
    assert_eq!(data["categories"], json!(["购物", "娱乐", "餐饮美食"]));
    let rows = data["data"].as_array().expect("data");
    assert_eq!(rows.len(), 15, "5 个月 × 3 个分类，含 0 值");
    assert_eq!(rows[0], json!(["2024-01", "购物", 40.0]));
    assert_eq!(rows[2], json!(["2024-01", "餐饮美食", 170.0]));
    assert_eq!(rows[6], json!(["2024-03", "购物", 0.0]));
}

#[test]
fn boxplot_points_keep_row_order_and_percentiles_are_linear() {
    let data = generate_boxplot_data(&dataset());
    assert_eq!(data["categories"], json!(["购物", "娱乐", "餐饮美食"]));

    let points = data["data"].as_array().expect("points");
    assert_eq!(points.len(), 24);
    assert_eq!(
        points[0],
        json!({"c": 0, "v": 10.0, "m": "便利店", "d": "2024-01-13"})
    );
    assert_eq!(
        points[23],
        json!({"c": 2, "v": 40.0, "m": "早市", "d": "2024-02-13"})
    );

    assert_eq!(
        data["box_data"],
        json!([
            [10.0, 30.0, 40.0, 237.5, 500.0],
            [10.0, 10.0, 15.0, 61.25, 200.0],
            [20.0, 20.0, 20.0, 25.0, 100.0],
        ])
    );
}

#[test]
fn heatmap_lists_hour_weekday_count_in_group_order() {
    let data = generate_heatmap_data(&dataset());
    let rows = data.as_array().expect("数组");
    assert_eq!(rows.len(), 22);
    assert_eq!(rows[0], json!([8, 0, 2]));
    assert_eq!(rows[5], json!([5, 1, 1]));
    assert_eq!(rows[21], json!([12, 6, 1]));
}

#[test]
fn pareto_cumulates_percentages() {
    let data = generate_pareto_data(&dataset());
    assert_eq!(
        data,
        json!({
            "categories": ["购物", "娱乐", "餐饮美食"],
            "values": [920.0, 475.0, 310.0],
            "percentages": [53.96, 81.82, 100.0],
        })
    );
}

#[test]
fn empty_dataset_returns_python_shapes() {
    let data = empty();
    assert_eq!(analyze_sankey(&data), json!({"nodes": [], "links": []}));
    assert_eq!(
        generate_chord_data(&data),
        json!({"nodes": [], "links": []})
    );
    assert_eq!(generate_funnel_data(&data), json!([]));
    assert_eq!(generate_quadrant_data(&data), json!([]));
    assert_eq!(
        generate_radar_data(&data),
        json!({"indicator": [], "series": []})
    );
    assert_eq!(generate_wordcloud_data(&data), json!([]));
    assert_eq!(
        generate_themeriver_data(&data),
        json!({"categories": [], "data": []})
    );
    let boxplot = generate_boxplot_data(&data);
    assert_eq!(boxplot, json!({"categories": [], "data": []}));
    assert!(boxplot.get("box_data").is_none(), "空集不输出 box_data");
    assert_eq!(generate_heatmap_data(&data), json!([]));
    assert_eq!(
        generate_pareto_data(&data),
        json!({"categories": [], "values": [], "percentages": []})
    );
}
