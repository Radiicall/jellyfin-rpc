# Jellyfin-RPC

<img src="https://shields.io/github/license/radiicall/jellyfin-rpc?color=purple"/> <img src="https://shields.io/github/v/tag/Radiicall/jellyfin-rpc"/> <img src="https://shields.io/github/downloads/radiicall/jellyfin-rpc/total"/>

[Frequently Asked Questions](FAQ.md)

Program used to display what you're currently watching on discord.

Jellyfin-RPC uses the API to check what you're currently watching, this means that the program can be ran from a server or your computer. The only requirement is that discord is open and logged in.


Example Movie:

![image](https://user-images.githubusercontent.com/66682497/213467832-5eb6b0a0-1b83-47db-bf00-48c0e739aec4.png)

Example Series:

![image](https://user-images.githubusercontent.com/66682497/213467669-8375841d-b846-4afe-8bd3-0b09f4c7f2ad.png)

Example Music:

![image](https://user-images.githubusercontent.com/66682497/228037565-56991219-2630-4da0-ae5a-b1fa904985de.png)

Example Live TV:

![image](https://user-images.githubusercontent.com/66682497/228035872-b6cdbf0a-ec6d-49b0-b238-c5ae9298943f.png)

Terminal Output:

![image](https://user-images.githubusercontent.com/66682497/222933540-aa5f08ed-afb2-4713-8b9a-18cbaa94444b.png)

## Setup
If you're using Windows then there's an install script that you can download <a href="https://github.com/Radiicall/jellyfin-rpc/raw/main/scripts/Auto-Install-win.bat">here</a>! (if anyone wants to make something similar for linux then make a PR!)

Make a `main.json` file with the following items in `$XDG_CONFIG_HOME/jellyfin-rpc` on Linux/macOS and `%APPDATA%\jellyfin-rpc\main.json` on Windows.

If you're unsure about the directory then run jellyfin-rpc and it will tell you where to place it.

```
{
    "Jellyfin": {
        "URL": "https://example.com",
        "API_KEY": "sadasodsapasdskd",
        "USERNAME": "your_username_here",
        "_comment": "the 2 lines below and this line arent needed and should be removed, by default nothing will display if these are present"
        "TYPE_BLACKLIST": ["music", "movie", "episode", "livetv"]
        "LIBRARY_BLACKLIST": ["Anime", "Anime Movies"]
    },
    "Discord": {
        "APPLICATION_ID": "1053747938519679018"
    },
    "Imgur": {
        "CLIENT_ID": "asdjdjdg394209fdjs093"
    },
    "Images": {
        "ENABLE_IMAGES": true,
        "IMGUR_IMAGES": true
    }
}
```

### Jellyfin
#### URL
This will be the URL to your jellyfin instance, remember to include http/https in the url.

If you want to know more about jellyfin you can check it out <a href="https://jellyfin.org/">here</a>.

#### API Key
This is the API key used for checking what you're currently watching on Jellyfin.

You can get one by going to \<YOUR INSTANCE URL HERE>/web/#!/apikeys.html

#### Username
This is the username you use to log into Jellyfin.

The username is needed because if you have multiple accounts (friends, family) then the program will just grab the first person it sees in the list.

### Discord Application ID
This step is optional as I have included my own.

If this variable is empty it will use the default one

You can make a discord application by going <a href="https://discord.com/developers/applications">here</a>.

### Imgur API

For the imgur api to work you have to do this in the config
```
"ENABLE_IMAGES": true,
"IMGUR_IMAGES": true
```

1. Go to Imgur's [application registration page](https://api.imgur.com/oauth2/addclient).
2. Enter any name for the application and pick OAuth2 without a callback URL as the authorisation type.
3. Submit the form to obtain your application's client ID.

Tutorial stolen from <a href="https://github.com/phin05/discord-rich-presence-plex#obtaining-an-imgur-client-id">discord-rich-presence-plex</a>

### Systemd

For systemd I have included <a href="https://raw.githubusercontent.com/Radiicall/jellyfin-rpc/main/scripts/jellyfin-rpc.service">this file</a>, you can download it directly by pressing ctrl+s on the page.

In the service file you have to change the `ExecStart=` line. You can launch the script without -c if you put the `main.json` file in the `XDG_CONFIG_HOME` directory

The service is supposed to run in user mode, put the service file into this directory `$HOME/.config/systemd/user/` and use `systemctl --user enable --now jellyfin-rpc.service` to start and enable it.

## Building
You need rust installed, you can get rustup from <a href="https://rustup.rs/">here</a>

If you already have rustup installed then make sure its the latest 2021 version, you can run `rustup update` to update to the newest version.

You also need openssl libs on linux (Don't remember exactly which one, running the jellyfin-rpc exec will tell you what you're missing)

Please make an issue with the missing libs in this repo so I can put them here, also say what distro you're running so I can test it.

After doing all of this you should be able to just run `cargo build` to get a binary.
In order to get an optimized binary just add `--release` to the end of cargo build.

Your binary will be located in `target/debug/jellyfin-rpc` or `target/release/jellyfin-rpc`
