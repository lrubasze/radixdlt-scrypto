#!/usr/bin/env python3

import logging
import subprocess
import time
from datetime import datetime
import os
from tabulate import tabulate
import click
import itertools

DRY_RUN = False
output_prefix = "output"

results = {}
tstamp = datetime.today().strftime("%Y-%m-%d_%H-%M-%S")

log = logging.getLogger()

def setup_logging(to_file=True):
    global log
    logFormatter = logging.Formatter("[%(asctime)s] [%(levelname)-5.5s] [%(funcName)-30s] %(message)s")

    if to_file:
        fileHandler = logging.FileHandler(f"{output_prefix}_{tstamp}.log")
        fileHandler.setFormatter(logFormatter)
        log.addHandler(fileHandler)

    consoleHandler = logging.StreamHandler()
    consoleHandler.setFormatter(logFormatter)

    log.addHandler(consoleHandler)
    log.setLevel(logging.INFO)


def set_result(step, variant, cmd, value):
    step += ";" + variant
    if results.get(step, None) is None:
        results[step] = {}

    if results[step].get(cmd, None) is None:
        results[step][cmd] = []

    results[step][cmd].append(value)


def exec_cmd(cmd, measure=True):
    if DRY_RUN:
        cmd = f"echo '{cmd}'"

    start_time = time.time()
    log.info(f"cmd:'{cmd}'")
    ret = subprocess.run(cmd, check=True, shell=True)
    ret.check_returncode()

    exec_time = None
    if measure:
        exec_time = round(time.time() - start_time, 3)
        log.info(f"cmd:'{cmd}' duration:{exec_time}s")
    return exec_time


def get_test_description(caption, rustc_wrapper, cargo_test_runner):
    return f"{caption} rustc_wrapper={rustc_wrapper} test_runner={cargo_test_runner}"


def do_clean():
    exec_cmd("./clean.sh", False)

def get_average(lst):
    return sum(lst) / len(lst)

def get_median(lst):
    l = lst.copy()
    l.sort()
    mid = len(l) // 2
    res = (l[mid] + l[~mid]) / 2
    return res

def print_results(results):
    table = []
    run_count = None
    hdr = ["Test", "command"]

    for d, cmds in results.items():
        # description consist of
        # - test name
        # - rustc wrapper info
        # - test runner info
        #d = d.split(" ")
        for cmd, vals in cmds.items():
            if run_count is None:
                run_count = len(vals)
                hdr.extend([f"run {c+1}" for c in range(run_count)])
                hdr.extend(["average", "median"])
            table.append([d, cmd, *vals, get_average(vals), get_median(vals)])

    click.echo("Test results:\n" + tabulate(table, headers=hdr))

    result_file = f"{output_prefix}_{tstamp}.csv"
    with open(result_file, "w") as fd:
        data = tabulate(table, headers=hdr, tablefmt="tsv")
        fd.write(data)
        click.echo(f"test results available in: {result_file}")

def parse_scenario(scenario):
    begin = False
    end = True
    variants = []
    commands = []
    steps = []
    options = {}

    sections = []

    for line in scenario:
        line = line.strip(" ").strip("\n")
        log.debug(f"{begin=} {end=} {len(commands)=} {len(steps)=} {line=}")

        if line == "#" or len(line) == 0:
            continue
        if line.startswith("# begin"):
            begin = True
            end = False
            variants = []
            commands = []
            steps = []
            options = {}

        if line.startswith("# name="):
            line = line[len("# name="):]
            steps.append(line.strip(" "))

        elif line.startswith("# variants"):
            # convert line to list of variants
            # example:
            #   below line
            #   # variants var1=opt1,opt2;var2=opt3,opt4
            #   should be converted to
            #   [ ['var1=opt1', 'var1=opt2'], ['var2=opt3', 'var2=opt4']]

            line = line[len("# variants"):]

            items = line.strip(" ").split(";")
            for i in items:
                k = i.split("=")[0]
                v = i.split("=")[1].split(",")

                # single- and double-quoted because it will evaluated into variable in the test script
                # example:
                #   var1=value with space
                #   ->
                #   'var1="value with space"'
                l = [f"'{k}=\"{j}\"'" for j in v]
                variants.append(l)

        elif line.startswith("# options"):
            # convert line to dictionary with options
            # example:
            #   # options opt1=val1;opt2=val2
            #   should be converted to
            #   { 'opt1' : 'val1', 'opt2' : 'val2'}
            line = line[len("# options"):]
            items = line.strip(" ").split(";")
            for i in items:
                option, value = i.split("=")
                options[option] = value

        elif line.startswith("# end"):
            end = True
            begin = False

            assert len(commands) == len(steps), f"number of commands not equal to steps, {commands=} {steps=}"

            log.info(f"{variants=} {commands=} {steps=} {options=}")

            sections.append([variants, commands, steps, options])

        elif line.startswith("#"):
            continue
        else:
            if end is not True:
                commands.append(line)
#            else:
#                exec_cmd(line, False)

    log.info(f"{len(sections)} sections found")
    return sections

def execute_scenario_section(variants, commands, steps, options):
    log.info(f"{len(variants)=} {len(commands)=} {len(steps)=} {options=}")

    for combinations in itertools.product(*variants):
        combinations = list(combinations)
        #log.info(f"{len(variants)=} {len(commands=)} {len(steps)=} {options=}")
        for i, cmd_step in enumerate(zip(commands, steps)):
            cmd, step = cmd_step
            cmd += " " + " ".join(combinations)
            log.info(f"### iteration {i+1}/{len(commands)} {step=} {cmd=}")

            if options.get('clean_before', False):
                do_clean()

            exec_time = exec_cmd(cmd, True)
            set_result(step, ";".join(combinations), cmd, exec_time)


def execute_scenario(sections, run_count):
    log.info(f"{len(sections)=} {run_count=}")

    for i in range(run_count):
        for j, s in enumerate(sections):
            log.info(f"## iteration {i+1}/{run_count} section {j+1}/{len(sections)} ")
            execute_scenario_section(*s)

def process_scenario(scenario, run_count):
    log.info(f"")

    sections = parse_scenario(scenario)

    execute_scenario(sections, run_count)

@click.group()
def main():
    pass

@click.command()
@click.argument(
    "scenario",
    type=click.File('r'),
)
@click.option("--run-count", default=1, show_default=True, help="Number of test runs")
@click.option("--out-prefix", default="output", show_default=True, help="Prefix of the output files")
@click.option(
    "--dry-run/--no-dry-run",
    default=False,
    show_default=True,
    help="Do not run real test if set to true",
)
def run(scenario, run_count, out_prefix, dry_run):
    """
    \b
    Run and measure duration of the tests specified in scenario file.

    It produces a CSV file output_*.csv with measurement results.
    """
    scenario = scenario.readlines()
    global DRY_RUN
    global output_prefix
    output_prefix = out_prefix

    setup_logging()

    if dry_run is True:
        DRY_RUN = True

    process_scenario(scenario, run_count)

    print_results(results)

@click.command()
@click.argument(
    "scenario",
    type=click.File('r'),
)
def parse(scenario):
    """
    \b
    Parse provided scenario file.
    """
    scenario = scenario.readlines()
    setup_logging()
    parse_scenario(scenario)

main.add_command(run)
main.add_command(parse)

if __name__ == "__main__":
    main()
