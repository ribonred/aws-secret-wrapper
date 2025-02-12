import os
import time
API_KEY = os.getenv("MY_API_KEY")
BB_SECRET = os.getenv("BB_SECRET")
HYPPY_API = os.getenv("HYPPY_API")
RABBIT_QUEUE = os.getenv("RABBIT_QUEUE")

while True:
    print("API_KEY:", API_KEY)
    print("BB_SECRET:", BB_SECRET)
    print("HYPPY_API:", HYPPY_API)
    print("RABBIT_QUEUE:", RABBIT_QUEUE)
    time.sleep(5)
