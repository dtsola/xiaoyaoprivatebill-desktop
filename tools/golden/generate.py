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


WEB_APP = None


def demo_frame():
    """按 Web 版 load_demo_data 的方式构造演示 DataFrame。"""
    df = pd.read_csv(DEMO)
    df["交易时间"] = pd.to_datetime(df["交易时间"])
    df["月份"] = df["交易时间"].dt.strftime("%Y-%m")
    df["日期"] = df["交易时间"].dt.strftime("%Y-%m-%d")
    if "是否退款" not in df.columns:
        df["是否退款"] = False
    if "来源" not in df.columns:
        df["来源"] = "示例数据"
    return df


def load_web_app(web_root: Path):
    """导入 Web 版 Flask app，并把数据源替换为演示数据。"""
    import sys

    sys.path.insert(0, str(web_root))

    # Web 版依赖 flask_session 做文件会话；生成 golden 只需要路由，
    # 用一个最小实现替代，避免额外安装依赖。
    import types

    if "flask_session" not in sys.modules:
        module = types.ModuleType("flask_session")

        class Session:  # noqa: D401
            def __init__(self, app=None):
                self.app = app

            def init_app(self, app):
                self.app = app

        module.Session = Session
        sys.modules["flask_session"] = module

    import app as web_app  # noqa: PLC0415
    import api.analysis as analysis_api  # noqa: PLC0415

    analysis_api.load_alipay_data = demo_frame
    return web_app.app


def build(case: dict) -> dict:
    name = case["endpoint"]
    if name == "/available_years":
        return available_years()
    if WEB_APP is None:
        raise SystemExit("该 case 需要 --web-root 指向 Web 版 backend 目录")
    client = WEB_APP.test_client()
    response = client.get(f"/api{name}", query_string=case.get("query", {}))
    payload = response.get_json()
    if payload is None:
        raise SystemExit(f"{name} 未返回 JSON（HTTP {response.status_code}）")
    return payload


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", required=True)
    parser.add_argument("--check", action="store_true", help="只比对，不写入")
    parser.add_argument("--web-root", help="Web 版 backend 目录（用于需要 Flask 的 case）")
    args = parser.parse_args()

    global WEB_APP
    if args.web_root:
        WEB_APP = load_web_app(Path(args.web_root))
    out_dir = Path(args.out).resolve()
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
