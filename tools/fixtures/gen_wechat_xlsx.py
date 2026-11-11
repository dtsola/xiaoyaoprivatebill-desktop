"""生成微信 XLSX 解析器测试夹具。

用法（需要 openpyxl）：
    python tools/fixtures/gen_wechat_xlsx.py

产物写入 fixtures/constructed/，内容稳定。
注意：xlsx 是 zip 容器，重复生成会改变内部时间戳，字节不完全一致；
内容变更时重新生成并提交，不要仅因字节差异反复提交。
"""

from datetime import datetime
from pathlib import Path

from openpyxl import Workbook

OUT_DIR = Path(__file__).resolve().parents[2] / "fixtures" / "constructed"

HEADER = [
    "交易时间",
    "交易类型",
    "交易对方",
    "商品",
    "收/支",
    "金额(元)",
    "支付方式",
    "当前状态",
    "交易单号",
    "商户单号",
    "备注",
]

PREAMBLE = [
    ["微信支付账单明细"],
    ["微信昵称：[小遥]"],
    ["起始时间：[2024-01-01 00:00:00] 终止时间：[2024-01-31 23:59:59]"],
    [],
    ["微信支付账单明细列表"],
]

ROWS = [
    [datetime(2024, 1, 15, 12, 30), "商户消费", "星巴克", "咖啡", "支出", 28.50, "零钱", "支付成功", "1001", "M001", ""],
    [datetime(2024, 1, 16, 8, 20), "交通出行", "地铁公司", "地铁票", "支出", 6.00, "零钱通", "支付成功", "1002", "M002", ""],
    [datetime(2024, 1, 17, 19, 0), "转账", "张三", "转账收款", "收入", 100.00, "零钱", "已退款", "1003", "M003", ""],
]

NO_AMOUNT_HEADER = ["交易时间", "交易类型", "交易对方", "商品", "收/支", "支付方式", "当前状态"]
NO_AMOUNT_ROWS = [
    [datetime(2024, 1, 15, 12, 30), "商户消费", "星巴克", "咖啡", "支出", "零钱", "支付成功"],
]

INVALID = [
    ["账单说明"],
    ["姓名", "金额(元)", "备注"],
    ["张三", 1.0, ""],
]


def build(name: str, header: list[str], rows: list[list]) -> None:
    workbook = Workbook()
    sheet = workbook.active
    sheet.title = "微信支付账单"
    for line in PREAMBLE:
        sheet.append(line)
    sheet.append(header)
    for row in rows:
        sheet.append(row)
    path = OUT_DIR / name
    workbook.save(path)
    print(f"已生成 {path}")


def main() -> None:
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    build("wechat_bill.xlsx", HEADER, ROWS)
    build("wechat_bill_no_amount.xlsx", NO_AMOUNT_HEADER, NO_AMOUNT_ROWS)

    workbook = Workbook()
    sheet = workbook.active
    for line in INVALID:
        sheet.append(line)
    path = OUT_DIR / "wechat_bill_invalid.xlsx"
    workbook.save(path)
    print(f"已生成 {path}")


if __name__ == "__main__":
    main()
