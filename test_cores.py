import multiprocessing, logging, os

logging.basicConfig(
    format="%(asctime)s - %(name)-8s - %(levelname)-8s - %(message)s",
    datefmt="%d-%b-%y %H:%M:%S",
)

try:
    # On Linux, we can detect how many cores are assigned to this process.
    # This is especially useful when running in a Docker container, when the
    # number of cores is intentionally limited.
    num_threads = len(os.sched_getaffinity(0))
except:
    # Default back to multiprocessing cpu_count, which is always going to count
    # the total number of cpus
    num_threads = multiprocessing.cpu_count()

print(f"Number of cores detected: {num_threads}")
