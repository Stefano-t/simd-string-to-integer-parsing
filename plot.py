import sys
import pandas as pd
import matplotlib.pyplot as plt

USAGE_STR = """
USAGE: python plot.py CSV [STAT]
Generate benchmark plots from the given csv file
   - CSV    path to the csv to plot
   - STAT   one of min, mean, max (default to min)
"""

def set_tsc_cycle_ax(ax, df, y1, y2, y3=None):
    ax.plot(df.index, df[y1], label=y1)
    ax.plot(df.index, df[y2], label=y2)
    if y3 is not None:
        ax.plot(df.index, df[y3], label=y3)
    ax.legend()
    ax.set_title("Latency")
    ax.set_ylabel("TSC cycles (lower is better)")
    ax.set_xlabel("Number length in bytes")
    ax.set_ylim(0)


def set_throughtput_ax(ax, df, y1, y2, y3=None):
    ax.plot(df.index, df.index / df[y1], label=y1)
    ax.plot(df.index, df.index / df[y2], label=y2)
    if y3 is not None:
        ax.plot(df.index, df.index / df[y3], label=y3)
    ax.legend()
    ax.set_title("Throughtput")
    ax.set_ylabel("Bytes per TSC cycle (higher is better)")
    ax.set_xlabel("Number length in bytes")
    ax.set_ylim(0)


def set_reciprocal_throughtput_ax(ax, df, y1, y2, y3=None):
    ax.plot(df.index, df[y1] / df.index, label=y1)
    ax.plot(df.index, df[y2] / df.index, label=y2)
    if y3 is not None:
        ax.plot(df.index, df[y3] / df.index, label=y3)
    ax.legend()
    ax.set_title("Reciprocal Throughtput")
    ax.set_ylabel("TSC cycles per byte (lower is better)")
    ax.set_xlabel("Number length in bytes")
    ax.set_ylim(0)


def plot_benchmark(df, y1, y2, y3=None):
    fig, (ax1, ax2, ax3) = plt.subplots(1, 3, dpi=80)
    set_tsc_cycle_ax(ax1, df, y1, y2, y3)
    set_throughtput_ax(ax2, df, y1, y2, y3)
    set_reciprocal_throughtput_ax(ax3, df, y1, y2, y3)
 
def print_usage():
    print(USAGE_STR, file=sys.stderr)

def get_column_to_plot(stat):
    if stat.lower() not in ("min", "mean", "max"):
        print_usage()
        print("ERROR: the provided statistic is not available.\n"
              "Please, choose one between min, mean, max.",
              file=sys.stderr)
        exit(1)
    
    return f"std_{stat.lower()} parse_integer_no_simd_{stat.lower()} std_delimeter_{stat.lower()} parse_integer_no_simd_delimeter_{stat.lower()} parse_integer_simd_delimeter_{stat.lower()}".split()

def main():
    argv_len = len(sys.argv)
    if argv_len < 2:
        print_usage()
        print("Error: no input file provided", file=sys.stderr)
        exit(1)
    csv = sys.argv[1]
    column_to_plot = []
    if argv_len == 3:
        column_to_plot = get_column_to_plot(sys.argv[2])
    else:
        column_to_plot = get_column_to_plot("min")

    df = pd.read_csv(csv, index_col=0)
    plot_benchmark(
        df,
        column_to_plot[0],
        column_to_plot[1],
    )
    plot_benchmark(
        df,
        column_to_plot[2],
        column_to_plot[3],
        column_to_plot[4],
    )
    plt.show()


if __name__ == "__main__":
    main()

