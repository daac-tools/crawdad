#!/usr/bin/env python3

import os
import sys
import csv
from argparse import ArgumentParser

if __name__ == "__main__":
    parser = ArgumentParser()
    parser.add_argument("lex_path")
    args = parser.parse_args()

    keys = set()
    with open(args.lex_path) as f:
        reader = csv.reader(f)
        for row in reader:
            keys.add(row[0])
    for key in sorted(keys):
        print(key)
