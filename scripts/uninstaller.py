import platform
import os
import subprocess
from time import sleep
import shutil

# Thanks to https://github.com/pogmommy for making the original macOS uninstaller
# Thanks to https://github.com/xenoncolt for making the original Windows uninstaller

print("Welcome to the Jellyfin-RPC uninstaller")
input("Hit enter to continue...")

if platform.system() != "Windows":
    if os.environ.get("XDG_CONFIG_HOME"):
        path = os.environ["XDG_CONFIG_HOME"].removesuffix("/") + "/jellyfin-rpc/"
    else:
        path = os.environ["HOME"].removesuffix("/") + "/.config/jellyfin-rpc/"
else:
    path = os.environ["APPDATA"].removesuffix("\\") + "\\jellyfin-rpc\\"

if platform.system() == "Windows":
    if os.path.isfile(path + "winsw.exe"):
        print("The script will ask for admin rights to remove the autostart service")
        print("waiting 5 seconds")
        sleep(5)
        subprocess.run([path + "winsw.exe", "uninstall"])

    shutil.rmtree(path)
elif platform.system() == "Darwin":
    if subprocess.run(["pgrep", "-xq", "--", "'jellyfin-rpc'"]).returncode == 0:
        subprocess.run(["killall", "jellyfin-rpc"])

    if "Jellyfin-RPC" in subprocess.Popen("launchctl list", shell=True, stdout=subprocess.PIPE).stdout.read().decode():
        subprocess.run(["launchctl", "remove", "Jellyfin-RPC"])

    servicepath = os.environ["HOME"].removesuffix("/") + "/Library/LaunchAgents/jellyfinrpc.local.plist"
    if os.path.isfile(servicepath):
        os.remove(servicepath)
    shutil.rmtree(path)
    os.remove("/usr/local/bin/jellyfin-rpc")
else:
    if "jellyfin-rpc.service" in subprocess.Popen("systemctl --user list-units", shell=True, stdout=subprocess.PIPE).stdout.read().decode():
        subprocess.run(["systemctl", "--user", "disable", "--now", "jellyfin-rpc.service"])

    if subprocess.run(["pgrep", "-xq", "--", "'jellyfin-rpc'"]).returncode == 0:
        subprocess.run(["killall", "jellyfin-rpc"])

    servicepath = path.removesuffix("jellyfin-rpc/") + "systemd/user/jellyfin-rpc.service"
    if os.path.isfile(servicepath):
        subprocess.run(["rm", servicepath])
    shutil.rmtree(path)
    os.remove(os.environ["HOME"].removesuffix("/") + "/.local/bin/jellyfin-rpc")

print("Uninstall complete!")
sleep(5)
