#!/usr/bin/env python3 

import os
from time import sleep

print("""
Welcome to the Jellyfin-RPC linux installer
[https://github.com/Radiicall/jellyfin-rpc#Setup]
""")

content = "{"

print("----------Jellyfin----------")
url = input("URL (include http/https): ")
api_key = input("API key: ")
username = input("username: ")

content += f' "jellyfin": {{ "url": "{url}", "api_key": "{api_key}",  "username": "{username}"'

print("If you dont want anything else you can just press enter through all of these")

while True:
    val = input("Do you want to customize music display? (y/N): ")

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
    val = input("Do you want to blacklist media types or libraries? (y/N): ")

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
    val = input("Do you want custom buttons? (y/N): ")

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
    val = input("Do you want images? (y/N): ")

    if val == "n" or val == "":
        break
    elif val != "y":
        print("Invalid input, please type y or n")
        continue

    val2 = input("Do you want imgur images? (y/N): ")
    client_id = ""

    if val2 == "y":
        client_id = input("Enter your imgur client id: ")
    elif val2 != "n" or val2 != "":
        print("Invalid input, please type y or n")
        continue

    if val2 == "y":
        content += f', "imgur": {{ "client_id": "{client_id}" }}, "images": {{ "enable_images": true, "imgur_images": true }}'
    else:
        content += f', "images": {{ "enable_images": true }}'

    break

content += " }"

path = ""

if os.environ.get("XDG_CONFIG_HOME"):
    path = os.environ["XDG_CONFIG_HOME"].removesuffix("/") + "/jellyfin-rpc/main.json"
else:
    path = os.environ["HOME"].removesuffix("/") + "/.config/jellyfin-rpc/main.json"

print(f"Placing config in '{path}'")

file = open(path, "w")
file.write(content)
file.close()
