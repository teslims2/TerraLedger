import time
import datetime

def poll_registry():
    print(f"[{datetime.datetime.now().isoformat()}] Polling carbon_registry for pending projects...")
    # Mock polling logic
    project_id = "P1"
    print(f"[{datetime.datetime.now().isoformat()}] Submitted monitoring for {project_id}")

if __name__ == "__main__":
    while True:
        poll_registry()
        time.sleep(6 * 3600)
