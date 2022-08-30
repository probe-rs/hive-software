import subprocess
from turtle import color
from semver import VersionInfo
import requests
from lxml import html
from yaspin import yaspin

HIVE_SOFTWARE_GITHUB_URL = "https://github.com/probe-rs/probe-rs/releases/latest"

def check_version():
    """Check if the currently installed version is older than the newest version. If not this function terminates the program."""

    result = None
    try:
        result = subprocess.run(["./monitor", "--version"], capture_output=True)
    except FileNotFoundError:
        print("Could not find any monitor installation in the folder this program was executed in. Please install the Hive testserver first using the 'install' subcommand")
        exit(1)
    except Exception as e:
        print(f"Something went wrong while checking the currently installed version {e}")
        exit(1)

    current_version_string_parts = result.stdout.decode("utf-8").split(" ")

    if len(current_version_string_parts) != 2:
        print("Failed to correctly identify the returned version of the currently installed monitor binary")
        exit(1)
    
    current_version = None
    try:
        current_version = VersionInfo.parse(current_version_string_parts[1])
    except Exception as e:
        print(f"Failed to parse extracted version of currently installed monitor binary: {e}")
        exit(1)

    print(current_version)


    if current_version.compare(get_latest_version()) >= 0:
        print("The currently installed version is already up to date. Exiting...")
        exit(0)


def get_latest_version():
    try:
        with yaspin(text="Getting latest version information...", color="blue"):
            res = requests.get(HIVE_SOFTWARE_GITHUB_URL)

        tree = html.document_fromstring(res.content)

        url_parts = tree.xpath("//head/meta[@property='og:url']/@content")[0].split("/")
        url_version: str = url_parts[len(url_parts) - 1]

        return VersionInfo.parse(url_version[1:])
    except Exception as e:
        print(f"Could not get the latest version information: {e}")
        exit(1)