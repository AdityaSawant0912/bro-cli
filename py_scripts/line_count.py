import os
import sys


def count_lines_in_directory(directory):
    if not os.path.isdir(directory):
        print(f"Error: '{directory}' is not a valid directory.")
        sys.exit(1)

    files = [f for f in os.listdir(directory) if os.path.isfile(os.path.join(directory, f))]

    if not files:
        print("No files found in the directory.")
        return

    print(f"{'File Name':<40} {'Lines':>10}")
    print("-" * 52)

    for filename in sorted(files):
        filepath = os.path.join(directory, filename)
        try:
            with open(filepath, "r", encoding="utf-8", errors="ignore") as f:
                line_count = sum(1 for _ in f)
            print(f"{filename:<40} {line_count:>10}")
        except Exception as e:
            print(f"{filename:<40} {'ERROR':>10}  ({e})")


if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Usage: python count_lines.py <directory_path>")
        sys.exit(1)

    count_lines_in_directory(sys.argv[1])