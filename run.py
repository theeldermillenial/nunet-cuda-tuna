import subprocess
import time
from pathlib import Path

import requests

# Grab all cuda code
git_url = "https://raw.githubusercontent.com/theeldermillenial/nunet-cuda-tuna/master/"
files = [
    "gpu_miner"
]

for file in files:
    response = requests.get(git_url+file)
    with open(file, "wb") as fw:
        fw.write(response.content)

# subprocess.run(["gcc", "-O3", "-v", "-c", "-o", "sha256.o", "sha256.c"])
# subprocess.run(["gcc", "-O3", "-v", "-c", "-o", "utils.o", "utils.c", "-lrt"])
# subprocess.run(["nvcc", "-O3", "-v", "-lrt", "-lm", "-o", "gpu_miner", "main.cu", "utils.o", "sha256.o"])

path = Path("gpu.log")
fw = open(path, "w")
process = subprocess.run(["chmod", "u+x", "gpu_miner"])
process = subprocess.Popen(["./gpu_miner"], stdout=subprocess.PIPE, text=True)

while True:

    response = requests.get("http://static.61.88.109.65.clients.your-server.de:8000/datum/")
    if response.status_code != 200:
        process.kill()
        quit()
    datum = response.text.split(",")[0]
    print("got datum...")
    with open("datum.txt", "w") as fw:
        fw.write(datum)
    start = time.time()

    time.sleep(10)
    if Path("submit.txt").exists():
        print("found a solution!")
        with open("submit.txt", "r") as fr:
            nonce = fr.read()
            response = requests.post("http://static.61.88.109.65.clients.your-server.de:8000/submit/", data=nonce, headers={"Content-Type":"text/plain"})
            print(nonce)

        Path("submit.txt").unlink()