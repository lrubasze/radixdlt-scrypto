#!/usr/bin/env python3

import logging
import subprocess
import time
from datetime import datetime
import os
from tabulate import tabulate
import click

SCCACHE_PATH = "/opt/homebrew/bin/sccache"

results = {}
tstamp = datetime.today().strftime("%Y-%m-%d_%H-%M-%S")

log = logging.getLogger()

cmds_with_std = [
    ["sbor", "cargo test"],
    ["sbor-derive", "cargo test"],
    ["sbor-tests", "cargo test"],
    ["scrypto", "cargo test"],
    ["scrypto", "cargo test --release"],
    ["scrypto-derive", "cargo test"],
    ["scrypto-tests", "cargo test"],
    ["radix-engine", "cargo test"],
    ["radix-engine", "cargo test --features wasmer"],
    ["transaction", "cargo test"],
]

cmds_with_no_std = [
    ["sbor", "cargo test --no-default-features --features alloc"],
    ["sbor-tests", "cargo test --no-default-features --features alloc"],
    ["scrypto", "cargo test --no-default-features --features alloc,prelude"],
    ["scrypto", "cargo test --no-default-features --features alloc,prelude --release"],
    ["scrypto-abi", "cargo test --no-default-features --features alloc"],
    ["scrypto-tests", "cargo test --no-default-features --features alloc"],
]

cmds_packages = [
    ["assets/blueprints/account", "scrypto test"],
    ["assets/blueprints/faucet", "scrypto test"],
    ["examples/hello-world", "scrypto test"],
    ["examples/no-std", "scrypto test"],
]

cmds_simulator = [
    ["simulator", "bash ./tests/resim.sh"],
    ["simulator", "bash ./tests/scrypto.sh"],
    ["simulator", "bash ./tests/manifest.sh"],
]

tests_benchmarks = [
    ["sbor-tests", "cargo bench"],
    ["radix-engine", "cargo bench"],
]

# Tests scenario list.
# Each list consist of tuple containing:
# - test list
# - test description
# - build-only flag, which says whether to:
#   - just build - True
#   - build and run - False
test_scenario = [
    (cmds_with_std, "with_std_build", True),
    (cmds_with_std, "with_std_run", False),
    (cmds_with_no_std, "with_no_std_build", True),
    (cmds_with_no_std, "with_no_std_run", False),
    (cmds_packages, "packages", False),
    (cmds_simulator, "simulator", False),
    (tests_benchmarks, "benchmarks", False),
]


def setup_logging():
    logFormatter = logging.Formatter("[%(asctime)s] [%(levelname)-5.5s]  %(message)s")

    fileHandler = logging.FileHandler(f"output_{tstamp}.log")
    fileHandler.setFormatter(logFormatter)
    log.addHandler(fileHandler)

    consoleHandler = logging.StreamHandler()
    consoleHandler.setFormatter(logFormatter)
    log.addHandler(consoleHandler)
    log.setLevel(logging.DEBUG)


def set_result(value, descr, folder="n/a", cmd="n/a"):
    if results.get(descr, None) is None:
        results[descr] = {}

    if results[descr].get(folder, None) is None:
        results[descr][folder] = {}

    if results[descr][folder].get(cmd, None) is None:
        results[descr][folder][cmd] = []

    results[descr][folder][cmd].append(value)


def exec_cmd(cmd, folder, descr, measure=True):
    start_time = time.time()
    cmd_str = " ".join(cmd)
    log.info(f"[{descr}] command:'{cmd_str}' folder:{folder}")
    subprocess.run(cmd, cwd=folder, check=True)
    exec_time = round(time.time() - start_time, 3)
    if measure:
        log.info(f"[{descr}] command:'{cmd_str}' duration:{exec_time}s")
        set_result(exec_time, descr, folder, cmd_str)


def setup_rustc_wrapper(rustc_wrapper):
    if rustc_wrapper == "sccache":
        os.environ["RUSTC_WRAPPER"] = SCCACHE_PATH
    elif rustc_wrapper == "none":
        if os.environ.get("RUSTC_WRAPPER", False):
            del os.environ["RUSTC_WRAPPER"]
    else:
        raise Exception(f"unknown rustc wrapper {rustc_wrapper} provided")


def setup_test_runner(cmd, cargo_test_runner):
    if cargo_test_runner != "test":
        if cmd[0] == "cargo" and cmd[1] == "test":
            cmd[1:2] = cargo_test_runner.split(" ")


def get_test_description(caption, rustc_wrapper, cargo_test_runner):
    return f"{caption} rustc_wrapper={rustc_wrapper} test_runner={cargo_test_runner}"


def tests_iterate(
    tests, caption, build_only=False, rustc_wrapper="", cargo_test_runner="test"
):
    start_time = time.time()

    descr = get_test_description(caption, rustc_wrapper, cargo_test_runner)
    log.info(f"== [{descr}] == ")

    for i, item in enumerate(tests):
        log.info(f"== [{descr}] == {i+1}/{len(tests)}")
        folder = item[0]
        cmd = item[1].split(" ")

        setup_rustc_wrapper(rustc_wrapper)
        setup_test_runner(cmd, cargo_test_runner)

        if build_only == True:
            cmd.append("--no-run")
        exec_cmd(cmd, folder, descr)

    exec_time = round(time.time() - start_time, 3)
    log.info(f"== [{descr}] == duration:{exec_time}s")
    set_result(exec_time, descr, "n/a")


def do_clean():
    exec_cmd(["./clean.sh"], ".", "Clean", False)


def print_results(results):
    table = []
    run_count = None
    hdr = ["Test", "rustc wrapper", "test runner", "folder", "command"]

    for d, folders in results.items():
        # description consist of
        # - test name
        # - rustc wrapper info
        # - test runner info
        d = d.split(" ")
        test_name = d[0]
        rustc_wrapper = d[1].replace('rustc_wrapper=','')
        test_runner = d[2].replace('test_runner=', '')
        for f, cmds in folders.items():
            for cmd, vals in cmds.items():
                if run_count is None:
                    run_count = len(vals)
                    hdr.extend(
                            [f"run {c+1}" for c in range(run_count)]
                        )
                table.append([test_name, rustc_wrapper, test_runner, f, cmd, *vals])
    log.info("\n" + tabulate(table, headers=hdr))

    result_file = f"output_{tstamp}.csv"
    with open(result_file, "w") as fd:
        data = tabulate(table, tablefmt="tsv")
        fd.write(data)
        log.info(f"test results available in: {result_file}")


@click.command()
@click.option("--run-count", default=1, help="Number of test runs")
def run(run_count):
    """
    \b
    Run and measure duration of the cargo tests in different configurations:
    - with rustc wrapper
    - with cargo nextest runner

    It produces a CSV file output_*.csv with measurement results.
    """
    for i in range(run_count):
        for rustc_wrapper in ["none", "sccache"]:
            for cargo_test_runner in ["test", "nextest run"]:
                descr = get_test_description(
                    f"test_cycle", rustc_wrapper, cargo_test_runner
                )

                log.info(f"### [{descr}] ### {i+1}/{run_count}")
                do_clean()

                start_time = time.time()

                for test_list, test_descr, build_only in test_scenario:
                    tests_iterate(
                        test_list,
                        test_descr,
                        build_only=build_only,
                        rustc_wrapper=rustc_wrapper,
                        cargo_test_runner=cargo_test_runner,
                    )

                exec_time = round(time.time() - start_time, 3)
                set_result(exec_time, descr)

                log.info(f"### [{descr}] ### {i+1}/{run_count} duration:{exec_time}s")

    print_results(results)


if __name__ == "__main__":
    setup_logging()
    run()
