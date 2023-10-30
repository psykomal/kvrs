import time
from locust import HttpUser, task, between


class RaftBenchmark(HttpUser):
    # wait_time = between(1, 5)
    # server1 = "127.0.0.1:21001"
    # server2 = "127.0.0.1:21002"
    # server3 = "127.0.0.1:21003"

    @task(3)
    def add_items(self):
        for item_id in range(100):
            res = self.client.post(
                f"/write",
                json={"Set": {"key": f"foo{item_id}", "value": f"bar{item_id}"}},
            )
            # print(res)

    @task
    def view_items(self):
        for item_id in range(100):
            res = self.client.post(
                f"/read",
                json=f"foo{item_id}",
            )
            # print(res)

    # def on_start(self):
    #     self.client.post(f"{server1}/init")
    #     self.client.post(f"{server1}/add-learner", json=[2, server2])
    #     self.client.post(f"{server1}/add-learner", json=[3, server3])
    #     self.client.post(f"{server1}/change-membership", json=[1, 2, 3])
