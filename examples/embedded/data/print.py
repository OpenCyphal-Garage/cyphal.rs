from os import environ
import pandas as pd
import numpy as np
import matplotlib.pyplot as plt

import argparse


def get_data_from_file(file):
    file = pd.read_csv("{}.csv".format(file))
    r = file.get("time")
    r = np.asarray(r)
    return r


def main():
    parser = argparse.ArgumentParser("print", description="print measure data")
    parser.add_argument("--heading", required=True, type=str,
                        help="heading of the final plot")
    parser.add_argument("--files", action="extend",
                        nargs="+", type=str, required=True)
    parser.add_argument("--min_y_range", nargs=2,
                        help="Range <from> <to> for the y axis. (If data of file out of this range, the range expands.)", required=False, type=int, action="extend")

    parser.add_argument("--data_lables", action="extend",
                        nargs="+", type=str, required=True, help="must be same amount as file list")

    args = parser.parse_args()

    plot_heading = args.heading if args.heading is not None else ""
    files = args.files
    y_range = sorted(
        args.min_y_range) if args.min_y_range is not None else None

    data_labels = args.data_lables

    if len(data_labels) != len(files):
        raise Exception("data lables list not equal to passed files")

    data_sets = list(map(get_data_from_file, files))

    plt.boxplot(data_sets, labels=data_labels)
    plt.ylabel('micros')

    data_range = (np.amin(data_sets) - 1, np.amax(data_sets) + 2)
    ticks = (y_range[0] if y_range[0] <= data_range[
        0] else data_range[0], y_range[1] + 1 if y_range[1] >= data_range[
        1] else data_range[1]) if y_range is not None else data_range
    plt.yticks(np.arange(ticks[0], ticks[1], step=1))

    plt.title(plot_heading)
    plt.show()


if __name__ == "__main__":
    main()
