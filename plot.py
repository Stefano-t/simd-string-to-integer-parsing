import sys
import pandas as pd
import matplotlib.pyplot as plt

USAGE_STR = """
USAGE: python plot.py CSV [STAT]
Generate benchmark plots from the given csv file
   - CSV    path to the csv to plot
   - STAT   one of min, mean, max (default to min)
"""


def set_tsc_cycle_ax(ax, df, y1, label1, y2, label2, y3=None, label3=None):
    ax.plot(df.index, df[y1], label=label1)
    ax.plot(df.index, df[y2], label=label2)
    if y3 is not None:
        ax.plot(df.index, df[y3], label=label3)
    ax.legend()
    ax.set_title("Latency")
    ax.set_ylabel("TSC cycles (lower is better)")
    ax.set_xlabel("Number length in bytes")
    ax.set_ylim(0)


def set_throughput_ax(ax, df, y1, label1, y2, label2, y3=None, label3=None):
    ax.plot(df.index, df.index / df[y1], label=label1)
    ax.plot(df.index, df.index / df[y2], label=label2)
    if y3 is not None:
        ax.plot(df.index, df.index / df[y3], label=label3)
    ax.legend()
    ax.set_title("Throughput")
    ax.set_ylabel("Bytes per TSC cycle (higher is better)")
    ax.set_xlabel("Number length in bytes")
    ax.set_ylim(0)


def set_reciprocal_throughput_ax(ax,
                                 df,
                                 y1,
                                 label1,
                                 y2,
                                 label2,
                                 y3=None,
                                 label3=None):
    ax.plot(df.index, df[y1] / df.index, label=label1)
    ax.plot(df.index, df[y2] / df.index, label=label2)
    if y3 is not None:
        ax.plot(df.index, df[y3] / df.index, label=label3)
    ax.legend()
    ax.set_title("Reciprocal Throughput")
    ax.set_ylabel("TSC cycles per byte (lower is better)")
    ax.set_xlabel("Number length in bytes")
    ax.set_ylim(0)


def plot_benchmark(df, y1, label1, y2, label2, y3=None, label3=None):
    fig, (ax1, ax2, ax3) = plt.subplots(1, 3, dpi=120)
    set_tsc_cycle_ax(ax1, df, y1, label1, y2, label2, y3, label3)
    set_throughput_ax(ax2, df, y1, label1, y2, label2, y3, label3)
    set_reciprocal_throughput_ax(ax3, df, y1, label1, y2, label2, y3, label3)


def print_usage():
    print(USAGE_STR, file=sys.stderr)


def get_column_to_plot(stat):
    if stat.lower() not in ("min", "mean", "max"):
        print_usage()
        print(
            "ERROR: the provided statistic is not available.\n"
            "Please, choose one between min, mean, max.",
            file=sys.stderr)
        exit(1)

    return f"std_{stat.lower()} parse_integer_no_simd_{stat.lower()} std_delimeter_{stat.lower()} parse_integer_no_simd_delimeter_{stat.lower()} parse_integer_simd_delimeter_{stat.lower()}".split(
    )


def main():
    argv_len = len(sys.argv)
    if argv_len < 2:
        print_usage()
        print("Error: no input file provided", file=sys.stderr)
        sys.exit(1)
    csv = sys.argv[1]
    column_to_plot = []
    task = "min"
    if argv_len == 3:
        task = sys.argv[2]
    column_to_plot = get_column_to_plot(task)

    df = pd.read_csv(csv, index_col=0)
    plot_benchmark(
        df,
        column_to_plot[0],
        f"naive method ({task})",
        column_to_plot[1],
        f"no simd library parsing ({task})",
    )
    plot_benchmark(
        df,
        column_to_plot[2],
        f"naive method ({task})",
        column_to_plot[3],
        f"no simd library parsing ({task})",
        column_to_plot[4],
        f"simd library parsing ({task})",
    )
    plt.tight_layout()
    plt.show()


if __name__ == "__main__":
    main()
