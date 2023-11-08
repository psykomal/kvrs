import time
import random
from locust import HttpUser, task, between


class RaftBenchmark(HttpUser):
    # wait_time = between(1, 5)
    # server1 = "127.0.0.1:21001"
    # server2 = "127.0.0.1:21002"
    # server3 = "127.0.0.1:21003"

    @task(3)
    def add_items(self):
        item_id = random.randint(1, 10000)
        res = self.client.get(
            f"/set",
            params={"key": f"foo{item_id}", "val": f"bar{item_id}"},
        )
        # print(res)

    @task
    def view_items(self):
        item_id = random.randint(1, 10000)
        res = self.client.get(
            f"/get",
            params={"key": f"foo{item_id}"},
        )
        # print(res)
