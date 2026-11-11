"""生成解析器 fixture 矩阵（P1-10）。

用法：python tools/fixtures/gen_parser_fixtures.py
产物写入 fixtures/constructed/。
"""

from pathlib import Path

OUT = Path(__file__).resolve().parents[2] / "fixtures" / "constructed"

ALIPAY_HEADER = "交易时间,交易分类,交易对方,商品说明,收/支,金额,收/付款方式,交易状态\n"
ALIPAY_ROWS = (
    "2024-01-15 12:30:00,餐饮美食,星巴克,咖啡,支出,28.50,余额宝,交易成功\n"
    "2024-01-16 09:00:00,转账,张三,退款,收入,100.00,余额宝,退款成功\n"
    "2024-01-17 10:00:00,餐饮美食,,,支出,15.00,余额宝,交易成功\n"
    "\n"
    "2024-01-18 11:00:00,交通出行,地铁公司,地铁票,支出,6.00,余额宝,交易成功\n"
)

WECHAT_HEADER = "交易时间,交易类型,交易对方,商品,收/支,金额(元),支付方式,当前状态\n"
WECHAT_ROWS = (
    "2024-01-15 12:30:00,商户消费,星巴克,咖啡,支出,¥28.50,零钱,支付成功\n"
    "2024-01-16 09:00:00,商户消费,便利店,早餐,支出,¥5.00,零钱,已退款\n"
)


def write(name: str, text: str, encoding: str) -> None:
    path = OUT / name
    path.write_bytes(text.encode(encoding))
    print(f"已生成 {path} ({encoding})")


def main() -> None:
    OUT.mkdir(parents=True, exist_ok=True)

    preamble = (
        "支付宝交易记录明细查询\n"
        "账号:[test@example.com]\n"
        "---------------------------------交易记录明细列表------------------------------------\n"
    )
    write(
        "alipay_gbk.csv",
        preamble + ALIPAY_HEADER + ALIPAY_ROWS + "----------------------\n导出时间:[2024-02-01 10:00:00]\n",
        "gbk",
    )
    write(
        "alipay_utf8_bom.csv",
        preamble + ALIPAY_HEADER + "2024-02-01 08:00:00,餐饮美食,星巴克,拿铁,支出,32.00,余额宝,交易成功\n",
        "utf-8-sig",
    )
    write(
        "wechat_utf8_bom.csv",
        "微信支付账单明细\n微信昵称：[小遥]\n\n" + WECHAT_HEADER + WECHAT_ROWS,
        "utf-8-sig",
    )
    write(
        "wechat_blank_lines.csv",
        "微信支付账单明细\n\n" + WECHAT_HEADER + "\n" + WECHAT_ROWS + "\n\n----------------------\n",
        "utf-8",
    )


if __name__ == "__main__":
    main()
