import csv
import random
import string
import os
from datetime import datetime, timedelta

# Output file and target size in bytes (2GB)
filename = "output.csv"
target_size = 20 * 1024 * 1024 * 1024  # 2GB


def random_string(length):
    return "".join(random.choices(string.ascii_letters, k=length))


def random_datetime():
    start = datetime(2000, 1, 1)
    end = datetime(2020, 12, 31)
    delta = end - start
    random_days = random.randint(0, delta.days)
    return (start + timedelta(days=random_days)).strftime("%Y-%m-%d %H:%M:%S")


# Define 50 column headers
header = []
for i in range(1, 51):
    if i % 5 == 1:
        header.append(f"string_{i}")
    elif i % 5 == 2:
        header.append(f"datetime_{i}")
    elif i % 5 == 3:
        header.append(f"int_{i}")
    elif i % 5 == 4:
        header.append(f"float_{i}")
    else:
        header.append(f"longtext_{i}")

with open(filename, "w", newline="", encoding="utf-8") as csvfile:
    writer = csv.writer(
        csvfile, delimiter=";", quotechar='"', quoting=csv.QUOTE_MINIMAL
    )
    writer.writerow(header)

    while os.path.getsize(filename) < target_size:
        row = []
        for i in range(1, 51):
            if i % 5 == 1:
                row.append(random_string(10))  # short string
            elif i % 5 == 2:
                row.append(random_datetime())  # datetime string
            elif i % 5 == 3:
                row.append(random.randint(0, 1000000))  # integer
            elif i % 5 == 4:
                row.append(round(random.uniform(0, 1000), 2))  # float
            else:
                row.append(random_string(50))  # long text
        writer.writerow(row)
