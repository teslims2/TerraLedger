import time

def push_benchmark_prices():
    prices = {
        "VCS": 15,
        "Gold Standard": 25,
        "ACR": 18
    }
    for methodology, price in prices.items():
        stroops = price * 10_000_000
        print(f"Pushing benchmark price for {methodology}: {price} USDC ({stroops} stroops)")

if __name__ == "__main__":
    while True:
        push_benchmark_prices()
        time.sleep(12 * 3600)
