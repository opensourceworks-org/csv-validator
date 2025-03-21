import csv
import random
import string
import os
from datetime import datetime, timedelta

# Output file and target size in bytes (2GB)
filename = "output.csv"
target_size = 2 * 1024 * 1024 * 1024  # 2GB

def random_string(length):
    return ''.join(random.choices(string.ascii_letters, k=length))

def random_datetime():
    start = datetime(2000, 1, 1)
    end = datetime(2020, 12, 31)
    delta = end - start
    random_days = random.randint(0, delta.days)
    return (start + timedelta(days=random_days)).strftime('%Y-%m-%d %H:%M:%S')

with open(filename, "w", newline='', encoding="utf-8") as csvfile:
    writer = csv.writer(csvfile, delimiter=';', quotechar='"', quoting=csv.QUOTE_MINIMAL)
    # Write header row
    writer.writerow(["string", "datetime", "int", "float", "longtext"])
    
    # Keep writing rows until the file size exceeds target_size
    while os.path.getsize(filename) < target_size:
        row = [
            random_string(10),         # short random string
            random_datetime(),         # random datetime as string
            random.randint(0, 1000000),  # random integer
            random.uniform(0, 1000),     # random float
            random_string(50)          # longer random text
        ]
        writer.writerow(row)
