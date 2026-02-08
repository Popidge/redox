#!/usr/bin/env python3
"""Generate LaTeX-ready phase-1 whitepaper figures."""

from __future__ import annotations

import argparse
import json
from pathlib import Path

import matplotlib.pyplot as plt
from matplotlib.figure import Figure


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Generate phase-1 figures for whitepaper"
    )
    parser.add_argument(
        "--repo-root",
        type=Path,
        default=Path(__file__).resolve().parents[1],
        help="Repository root (default: inferred from script location)",
    )
    parser.add_argument(
        "--out-dir",
        type=Path,
        default=Path("docs/figures/phase1"),
        help="Output directory for figure artifacts",
    )
    return parser.parse_args()


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def save_dual(fig: Figure, out_base: Path) -> None:
    out_base.parent.mkdir(parents=True, exist_ok=True)
    fig.savefig(out_base.with_suffix(".pdf"), bbox_inches="tight")
    fig.savefig(out_base.with_suffix(".png"), dpi=220, bbox_inches="tight")


def make_money_plot(repo_root: Path, out_dir: Path) -> None:
    report = load_json(repo_root / "eval/v2_6/report_aggregate_2seeds.json")

    rust_compile = report["summary"]["rust"]["compile_at_1"]["mean"]
    rust_test = report["summary"]["rust"]["test_at_1"]["mean"]
    iron_compile = report["summary"]["iron"]["compile_at_1"]["mean"]
    iron_test = report["summary"]["iron"]["test_at_1"]["mean"]

    metrics = ["compile@1", "test@1"]
    rust_vals = [rust_compile, rust_test]
    iron_vals = [iron_compile, iron_test]

    x = range(len(metrics))
    width = 0.36

    fig, ax = plt.subplots(figsize=(7.8, 4.8))
    rust_bars = ax.bar(
        [i - width / 2 for i in x],
        rust_vals,
        width,
        label="Rust arm",
        color="#4C78A8",
    )
    iron_bars = ax.bar(
        [i + width / 2 for i in x],
        iron_vals,
        width,
        label="Iron arm",
        color="#F58518",
    )

    ax.set_ylim(0.0, 1.05)
    ax.set_ylabel("Rate")
    ax.set_xticks(list(x), metrics)
    ax.set_title("Final Performance (v2.6 aggregate, 2 seeds)")
    ax.grid(axis="y", linestyle="--", alpha=0.35)
    ax.legend(frameon=False, loc="upper left")

    for bars in (rust_bars, iron_bars):
        for bar in bars:
            h = bar.get_height()
            ax.annotate(
                f"{h:.3f}",
                xy=(bar.get_x() + bar.get_width() / 2, h),
                xytext=(0, 4),
                textcoords="offset points",
                ha="center",
                va="bottom",
                fontsize=9,
            )

    save_dual(fig, out_dir / "fig1_money_plot_v26")
    plt.close(fig)


def make_progression_plot(repo_root: Path, out_dir: Path) -> None:
    points = [
        (
            "v1",
            load_json(repo_root / "eval/v1/report_aggregate_2seeds.json")["summary"][
                "iron"
            ]["test_at_1"]["mean"],
        ),
        (
            "v2",
            load_json(repo_root / "eval/v2/report_seed2108.json")["iron"]["test_at_1"],
        ),
        (
            "v2.5",
            load_json(repo_root / "eval/v2_5/report_aggregate_2seeds.json")["summary"][
                "iron"
            ]["test_at_1"]["mean"],
        ),
        (
            "v2.6",
            load_json(repo_root / "eval/v2_6/report_aggregate_2seeds.json")["summary"][
                "iron"
            ]["test_at_1"]["mean"],
        ),
    ]

    labels = [p[0] for p in points]
    vals = [p[1] for p in points]
    x_vals = list(range(len(labels)))

    fig, ax = plt.subplots(figsize=(7.8, 4.8))
    ax.plot(x_vals, vals, marker="o", linewidth=2.4, color="#F58518")
    ax.fill_between(x_vals, vals, [0] * len(vals), alpha=0.09, color="#F58518")
    ax.set_xticks(x_vals, labels)

    ax.set_ylim(0.0, 1.05)
    ax.set_ylabel("Iron test@1")
    ax.set_title("Iterative Improvement of Iron Functional Correctness")
    ax.grid(axis="y", linestyle="--", alpha=0.35)

    for x, y in zip(x_vals, vals):
        ax.annotate(
            f"{y:.3f}",
            xy=(x, y),
            xytext=(0, 7),
            textcoords="offset points",
            ha="center",
            va="bottom",
            fontsize=9,
        )

    save_dual(fig, out_dir / "fig2_progression_iron_test")
    plt.close(fig)


def make_error_shift_plot(repo_root: Path, out_dir: Path) -> None:
    report = load_json(repo_root / "eval/v2_5/report_aggregate_2seeds.json")
    rust_tax = report["error_taxonomy"]["rust"]
    iron_tax = report["error_taxonomy"]["iron"]

    categories = ["type_mismatch", "iron_parse_or_oxidize", "other", "name_resolution"]
    colors = {
        "type_mismatch": "#4C78A8",
        "iron_parse_or_oxidize": "#E45756",
        "other": "#72B7B2",
        "name_resolution": "#54A24B",
    }

    arms = ["Rust", "Iron"]
    values = {
        "Rust": {c: rust_tax.get(c, 0) for c in categories},
        "Iron": {c: iron_tax.get(c, 0) for c in categories},
    }

    fig, ax = plt.subplots(figsize=(7.8, 4.8))
    bottoms = [0, 0]

    for cat in categories:
        segment = [values[arm][cat] for arm in arms]
        ax.bar(
            arms,
            segment,
            bottom=bottoms,
            label=cat,
            color=colors[cat],
        )
        bottoms = [b + s for b, s in zip(bottoms, segment)]

    ax.set_ylabel("Failure count")
    ax.set_title("Failure Taxonomy Contrast (v2.5 aggregate)")
    ax.grid(axis="y", linestyle="--", alpha=0.35)
    ax.legend(frameon=False, loc="upper right")

    save_dual(fig, out_dir / "fig3_error_shift_v25")
    plt.close(fig)


def make_syntax_listing(repo_root: Path, out_dir: Path) -> None:
    rust_code = (
        (repo_root / "data/pilot/foundation_v2/rust/closure_shift_001.rs")
        .read_text(encoding="utf-8")
        .strip()
    )

    iron_code = """function closure_shift_001
    takes x of i32
    returns i32
begin
    define f as closure with parameters n and body n plus 3
    call f with x
end function"""

    tex = f"""% Figure 4: syntax comparison using lstlisting
% Requires packages: listings, caption

\\begin{{figure}}[t]
\\centering
\\begin{{minipage}}[t]{{0.48\\linewidth}}
\\captionof{{lstlisting}}{{Rust snippet}}
\\begin{{lstlisting}}[language=Rust,basicstyle=\\ttfamily\\small]
{rust_code}
\\end{{lstlisting}}
\\end{{minipage}}\\hfill
\\begin{{minipage}}[t]{{0.48\\linewidth}}
\\captionof{{lstlisting}}{{Iron snippet}}
\\begin{{lstlisting}}[basicstyle=\\ttfamily\\small]
{iron_code}
\\end{{lstlisting}}
\\end{{minipage}}
\\caption{{Syntax comparison: Rust vs Iron for the same function contract.}}
\\label{{fig:syntax_comparison}}
\\end{{figure}}
"""

    (out_dir / "fig4_syntax_comparison.tex").write_text(tex, encoding="utf-8")


def make_include_snippets(out_dir: Path) -> None:
    snippets = """% Figure include snippets for LaTeX
% Add in preamble: \\usepackage{graphicx}, \\usepackage{listings}, \\usepackage{caption}

% Figure 1: Money Plot
\\begin{figure}[t]
  \\centering
  \\includegraphics[width=0.82\\linewidth]{docs/figures/phase1/fig1_money_plot_v26.pdf}
  \\caption{Final performance comparison (v2.6 aggregate): compile@1 and test@1.}
  \\label{fig:money_plot}
\\end{figure}

% Figure 2: Progression
\\begin{figure}[t]
  \\centering
  \\includegraphics[width=0.82\\linewidth]{docs/figures/phase1/fig2_progression_iron_test.pdf}
  \\caption{Iron test@1 progression across iterations (v1 to v2.6).}
  \\label{fig:progression}
\\end{figure}

% Figure 3: Error Shift
\\begin{figure}[t]
  \\centering
  \\includegraphics[width=0.82\\linewidth]{docs/figures/phase1/fig3_error_shift_v25.pdf}
  \\caption{Failure taxonomy contrast (v2.5 aggregate): Rust semantic failures vs Iron transform failures.}
  \\label{fig:error_shift}
\\end{figure}

% Figure 4 uses a listing-based figure snippet:
% \\input{docs/figures/phase1/fig4_syntax_comparison.tex}
"""
    (out_dir / "latex_figure_snippets.tex").write_text(snippets, encoding="utf-8")


def main() -> int:
    args = parse_args()
    repo_root = args.repo_root.resolve()
    out_dir = (repo_root / args.out_dir).resolve()
    out_dir.mkdir(parents=True, exist_ok=True)

    make_money_plot(repo_root, out_dir)
    make_progression_plot(repo_root, out_dir)
    make_error_shift_plot(repo_root, out_dir)
    make_syntax_listing(repo_root, out_dir)
    make_include_snippets(out_dir)

    print(f"Wrote figure artifacts to {out_dir}")
    print("- fig1_money_plot_v26.pdf/png")
    print("- fig2_progression_iron_test.pdf/png")
    print("- fig3_error_shift_v25.pdf/png")
    print("- fig4_syntax_comparison.tex")
    print("- latex_figure_snippets.tex")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
