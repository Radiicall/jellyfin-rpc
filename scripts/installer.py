#!/usr/bin/env python3

# Thanks to https://github.com/pogmommy for making the original macOS installer
# Thanks to https://github.com/xenoncolt for making the original Windows installer
# Their contributions made this universal script a lot easier to produce.

import os
import subprocess
import platform
from time import sleep

path = ""

if platform.system() != "Windows":
    if os.environ.get("XDG_CONFIG_HOME"):
        path = os.environ["XDG_CONFIG_HOME"].removesuffix("/") + "/jellyfin-rpc/main.json"
    else:
        path = os.environ["HOME"].removesuffix("/") + "/.config/jellyfin-rpc/main.json"

    subprocess.run(["mkdir", "-p", path.removesuffix("main.json")])
else:
    path = os.environ["APPDATA"].removesuffix("\\") + "\jellyfin-rpc\main.json"
    subprocess.run(["powershell", "-Command", f'mkdir "{path.removesuffix("main.json")}"'], stdout=subprocess.DEVNULL)

print("""
Welcome to the Jellyfin-RPC installer
[https://github.com/Radiicall/jellyfin-rpc#Setup]
""")

current = ""

if os.path.isfile(path):
    print(f"Found existing config: {path}")
    while True:
        current = input("Use existing config? (y/N): ").lower()
        if current == "n" or current == "y" or current == "":
            break
        print("Invalid input, please type y or n")

if current == "n" or current == "":
    content = "{"

    print("----------Jellyfin----------")
    url = input("URL (include http/https): ")
    api_key = input(f"API key [Create one here: {url}/web/index.html#!/apikeys.html]: ")
    username = input("username: ")

    content += f' "jellyfin": {{ "url": "{url}", "api_key": "{api_key}",  "username": "{username}"'

    print("If you dont want anything else you can just press enter through all of these")

    while True:
        val = input("Do you want to customize music display? (y/N): ").lower()

        if val == "n" or val == "":
            break
        elif val != "y":
            print("Invalid input, please type y or n")
            continue

        print("Enter what you would like to be shown in a comma seperated list")
        print("Remember that it will show in the order you type it in")
        print("Valid options are year, album and/or genres")
        display = input("[Default: genres]: ")

        print("Choose the separator between the artist name and the info")
        separator = input("[Default: -]: ")

        if display != "" and separator != "":
            content += f', "music": {{ "display": "{display}", "separator": "{separator}" }}'
        elif display != "" and separator == "":
            content += f', "music": {{ "display": "{display}" }}'
        elif display == "" and separator != "":
            content += f', "music": {{ "separator": "{separator}" }}'

        break

    while True:
        val = input("Do you want to blacklist media types or libraries? (y/N): ").lower()

        if val == "n" or val == "":
            content += " }"
            break
        elif val != "y":
            print("Invalid input, please type y or n")
            continue

        print("You will first type in what media types to blacklist, this should be a comma separated list WITHOUT SPACES")
        print("then after that you can choose what libraries to blacklist, this should ALSO be a comma separated list,")
        print("there should be no spaces before or after the commas but there can be spaces in the names of libraries")
        sleep(2)
        
        print("Media types 1/2")
        media_types = input("Valid types are music, movie, episode and/or livetv [Default: ]: ").split(",")

        print("Libraries 2/2")
        libraries = input("Enter libraries to blacklist [Default: ]: ").split(",")

        content += ', "blacklist": { "media_types": [ '
        for i in media_types:
            content += f'"{i}", '

        content = content.removesuffix(", ")
        
        content += ' ], "libraries": ['
        for i in libraries:
            content += f'"{i}", '

        content = content.removesuffix(", ")
        content += " ] } }"

        break

    print("----------Discord----------")

    content += ', "discord": {'

    appid = input("Enter your discord application ID [Default: 1053747938519679018]: ")
    if appid != "":
        content += f' "application_id": "{appid}"'

    while True:
        val = input("Do you want custom buttons? (y/N): ").lower()

        if val == "n" or val == "":
            content += " }"
            break
        elif val != "y":
            print("Invalid input, please type y or n")
            continue

        if appid != "":
            content += ","
        
        content += ' "buttons": [ '

        print("If you want one button to continue being dynamic then you have to specifically enter dynamic into both fields")
        print("If you dont want any buttons to appear then you can leave everything blank here and it wont show anything.")

        print("Button 1/2")
        name = input("Choose what the button will show [Default: dynamic]: ")
        url = input("Choose where the button will direct to [Default: dynamic]: ")

        button1 = False
        if name != "" and url != "":
            content += f'{{ "name": "{name}", "url": "{url}" }}'
            button1 = True

        print("Button 2/2")
        name = input("Choose what the button will show [Default: dynamic]: ")
        url = input("Choose where the button will direct to [Default: dynamic]: ")

        if name != "" and url != "" and button1 == True:
            content += ", "
        if name != "" and url != "":
            content += f'{{ "name": "{name}", "url": "{url}" }}'

        content += " ] }"
        break

    print("----------Images----------")

    while True:
        val = input("Do you want images? (y/N): ").lower()

        if val == "n" or val == "":
            break
        elif val != "y":
            print("Invalid input, please type y or n")
            continue

        val2 = input("Do you want imgur images? (y/N): ").lower()
        client_id = ""

        if val2 == "y":
            client_id = input("Enter your imgur client id: ")
        elif val2 != "n" and val2 != "":
            print("Invalid input, please type y or n")
            continue

        if val2 == "y":
            content += f', "imgur": {{ "client_id": "{client_id}" }}, "images": {{ "enable_images": true, "imgur_images": true }}'
        else:
            content += f', "images": {{ "enable_images": true }}'

        break

    content += " }"

    print(f"\nPlacing config in '{path}'")

    file = open(path, "w")
    file.write(content)
    file.close()
    

print("\nDownloading Jellyfin-RPC")

if platform.system() == "Windows":
    path = path.removesuffix("main.json")
    subprocess.run(["curl", "-o", path + "jellyfin-rpc.exe", "-L", "https://github.com/Radiicall/jellyfin-rpc/releases/latest/download/jellyfin-rpc.exe"])
    while True:
        val = input("Do you want to autostart Jellyfin-RPC at login? (y/N): ").lower()

        if val == "n" or val == "":
            break
        if val != "y":
            print("Invalid input, please type y or n")
            continue

        if os.path.isfile(path + "winsw.exe"):
            print("The script will prompt for administrator to remove the already installed service")
            sleep(1)
            subprocess.run[path + "winsw.exe", "uninstall"]

        subprocess.run(["curl", "-o", path + "winsw.exe", "-L", "https://github.com/winsw/winsw/releases/latest/download/WinSW-x64.exe"])

        content = f"""<service>
    <id>jellyfin-rpc</id>
    <name>Jellyfin-RPC</name>
    <description>This service is running Jellyfin-RPC for rich presence support</description>
    <executable>{path}jellyfin-rpc.exe</executable>
    <arguments>-c {path}main.json -i {path}urls.json</arguments>
</service>"""

        file = open(path + "winsw.xml", "w")
        file.write(content)
        file.close()

        print("The program will now ask you for administrator rights twice, this is so the service can be installed!")
        print("waiting 5 seconds")
        sleep(5)

        subprocess.run([path + "winsw.exe", "install"])
        subprocess.run([path + "winsw.exe", "start"])

        print("Autostart has been set up, jellyfin-rpc should now launch at login\nas long as there are no issues with the configuration")

        break

elif platform.system() == "Darwin":
    file = f"https://github.com/Radiicall/jellyfin-rpc/releases/latest/download/jellyfin-rpc-{platform.machine()}-linux"
    subprocess.run(["curl", "-o", "/usr/local/bin/jellyfin-rpc", "-L", file])
    subprocess.run(["chmod", "+x", "/usr/local/bin/jellyfin-rpc"])

    while True:
        val = input("Do you want to autostart Jellyfin-RPC at login? (y/N): ").lower()

        if val == "n" or val == "":
            break
        if val != "y":
            print("Invalid input, please type y or n")
            continue

        if subprocess.run(["pgrep", "-xq", "--", "'jellyfin-rpc'"]).returncode == 0:
            subprocess.run(["killall", "jellyfin-rpc"])

        if "Jellyfin-RPC" in subprocess.Popen("launchctl list", shell=True, stdout=subprocess.PIPE).stdout.read().decode():
            subprocess.run(["launchctl", "remove", "Jellyfin-RPC"])

        content = """<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>Jellyfin-RPC</string>
    <key>Program</key>
    <string>/usr/local/bin/jellyfin-rpc</string>
    <key>RunAtLoad</key>
    <true/>
    <key>StandardErrorPath</key>
    <string>/tmp/jellyfinrpc.local.stderr.txt</string>
    <key>StandardOutPath</key>
    <string>/tmp/jellyfinrpc.local.stdout.txt</string>
</dict>
</plist>"""

        path = os.environ["HOME"] + "/Library/LaunchAgents/jellyfinrpc.local.plist"

        file = open(path, "w")
        file.write(content)
        file.close()

        subprocess.run(["chmod", "644", path])
        subprocess.run(["launchctl", "load", path])

        print("Jellyfin RPC is now set up to start at login.")
        print("If needed, you can run Jellyfin RPC at any time by running 'jellyfin-rpc' in a terminal.")
        break
else:
    subprocess.run(["mkdir", "-p", os.environ["HOME"].removesuffix("/") + "/.local/bin"])
    subprocess.run(["curl", "-o", os.environ["HOME"].removesuffix("/") + "/.local/bin/jellyfin-rpc", "-L", "https://github.com/Radiicall/jellyfin-rpc/releases/latest/download/jellyfin-rpc-x86_64-linux"])
    subprocess.run(["chmod", "+x", os.environ["HOME"].removesuffix("/") + "/.local/bin/jellyfin-rpc"])

    if os.environ.get("XDG_CONFIG_HOME"):
        path = os.environ["XDG_CONFIG_HOME"].removesuffix("/") + "/systemd/user/jellyfin-rpc.service"
    else:
        path = os.environ["HOME"].removesuffix("/") + "/.config/systemd/user/jellyfin-rpc.service"

    while True:
        val = input("Do you want to autostart Jellyfin-RPC at login using Systemd? (y/N): ").lower()

        if val == "n" or val == "":
            break
        if val != "y":
            print("Invalid input, please type y or n")
            continue

        print(f"\nSetting up service file in {path}")

        subprocess.run(["mkdir", "-p", path.removesuffix("jellyfin-rpc.service")])

        content = f"""[Unit]
Description=Jellyfin-RPC Service
Documentation=https://github.com/Radiicall/jellyfin-rpc
After=network.target

[Service]
Type=simple
ExecStart={os.environ["HOME"].removesuffix("/") + "/.local/bin/jellyfin-rpc"}
Restart=on-failure

[Install]
WantedBy=default.target"""

        file = open(path, "w")
        file.write(content)
        file.close()

        subprocess.run(["systemctl", "--user", "daemon-reload"])
        subprocess.run(["systemctl", "--user", "enable", "--now", "jellyfin-rpc.service"])

        print("Jellyfin-RPC is now set up to start at login.")

        break

print("Installation complete!")

