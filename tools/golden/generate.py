"""生成 golden 期望 JSON。

用法：
    python tools/golden/generate.py --out fixtures/expected
    python tools/golden/generate.py --out fixtures/expected --check

只依赖 pandas，不启动 Flask；每个 case 复刻对应路由的返回体。
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

import pandas as pd

ROOT = Path(__file__).resolve().parents[2]
DEMO = ROOT / "crates" / "bill-services" / "data" / "sample_data.csv"
CASES = ROOT / "fixtures" / "golden" / "cases.json"


def available_years() -> dict:
    df = pd.read_csv(DEMO)
    df["交易时间"] = pd.to_datetime(df["交易时间"])
    years = sorted(df["交易时间"].dt.year.unique().tolist(), reverse=True)
    return {"success": True, "years": [int(year) for year in years]}


def build(case: dict) -> dict:
    name = case["endpoint"]
    if name == "/available_years":
        return available_years()
    raise SystemExit(f"未实现的 case: {name}")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", required=True)
    parser.add_argument("--check", action="store_true", help="只比对，不写入")
    args = parser.parse_args()

    out_dir = Path(args.out)
    out_dir.mkdir(parents=True, exist_ok=True)
    cases = json.loads(CASES.read_text(encoding="utf-8"))

    failed = False
    for case in cases["cases"]:
        payload = build(case)
        text = json.dumps(payload, ensure_ascii=False, indent=2, sort_keys=True) + "\n"
        target = out_dir / case["expected"]
        if args.check:
            current = target.read_text(encoding="utf-8") if target.exists() else ""
            if current != text:
                print(f"[FAIL] {target} 与 Python 输出不一致", file=sys.stderr)
                failed = True
            else:
                print(f"[OK] {target}")
        else:
            target.write_text(text, encoding="utf-8")
            print(f"已生成 {target}")

    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())
