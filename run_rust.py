import json
import subprocess
import time
from dataclasses import dataclass
from datetime import datetime, timedelta
from pathlib import Path

import requests
import pycardano


@dataclass
class State(pycardano.PlutusData):
    CONSTR_ID = 0
    nonce: bytes
    block_number: int
    current_hash: bytes
    leading_zeroes: int
    difficulty_number: int
    epoch_time: int


def get_datum():
    response = requests.get(
        "https://piefayth.dev/pool/work",
        params={
            "address": "addr1q9zs05ya2wqv2ftzqgckcmfg0xq7s9cnp99eyj2dcqaxyx96h2p5jcgwnv4tw5tq3yzd2dmh3sgcgfyta3tv8x3vdq8qvuzr64"
        },
    ).json()

    state_vals = {
        "nonce",
        "block_number",
        "current_hash",
        "leading_zeroes",
        "difficulty_number",
        "epoch_time",
    }

    state_dict = {
        key: value
        for key, value in response["current_block"].items()
        if key in state_vals
    }
    state_dict["nonce"] = bytes.fromhex(response["nonce"])
    state_dict["current_hash"] = bytes.fromhex(state_dict["current_hash"])

    state = State(**state_dict)

    return state.to_cbor_hex(), response["miner_id"]


# Grab all cuda code
git_url = "https://raw.githubusercontent.com/theeldermillenial/nunet-cuda-tuna/master/"
files = ["rminer"]

for file in files:
    response = requests.get(git_url + file)
    with open(file, "wb") as fw:
        fw.write(response.content)

datum, miner_id = get_datum()
with open("datum.txt", "w") as fw:
    fw.write(datum)

process = subprocess.run(["chmod", "u+x", "rminer"])
process = subprocess.Popen(["./rminer"], stdout=subprocess.PIPE, text=True)
while True:
    response = requests.get(
        "http://static.61.88.109.65.clients.your-server.de:8000/datum/"
    )
    if response.status_code != 200:
        process.kill()
        quit()
    new_datum, miner_id = get_datum()
    if new_datum != datum:
        print("got new datum...")
        datum = new_datum
        with open("datum.txt", "w") as fw:
            fw.write(datum)
    start = time.time()
    nonces = []
    while time.time() - start < 30:
        if Path("submit.txt").exists():
            with open("submit.txt", "r") as fr:
                for line in fr:
                    nonces.append(line.rstrip("\n"))
                Path("submit.txt").unlink()
    if len(nonces) > 0:
        print(f"Found {len(nonces)} nonces: {nonces}")

        response = requests.post(
            "https://piefayth.dev/pool/submit",
            headers={"Content-type": "application/json"},
            data=json.dumps(
                {
                    "address": "addr1q9zs05ya2wqv2ftzqgckcmfg0xq7s9cnp99eyj2dcqaxyx96h2p5jcgwnv4tw5tq3yzd2dmh3sgcgfyta3tv8x3vdq8qvuzr64",
                    "entries": [{"nonce": nonce} for nonce in nonces],
                }
            ),
        )
        print(f"Successfully submitted {response.json()['num_accepted']} responses!")

    response = requests.get(
        "https://piefayth.dev/pool/hashrate",
        params={
            "miner_id": miner_id,
            "start_time": int((datetime.now() - timedelta(seconds=900)).timestamp()),
        },
    )
    print(f"Hash rate (hourly average): {response.text}")
