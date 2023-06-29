# Jellyfin-RPC

<img src="https://shields.io/github/license/radiicall/jellyfin-rpc?color=purple"/> <img src="https://shields.io/github/v/tag/Radiicall/jellyfin-rpc"/> <img src="https://shields.io/github/downloads/radiicall/jellyfin-rpc/total"/>

[Frequently Asked Questions](FAQ.md)

Program used to display what you're currently watching on discord.

Jellyfin-RPC uses the API to check what you're currently watching, this means that the program can be ran from a server or your computer. The only requirement is that discord is open and logged in.

<details>
  <summary>Pictures of Jellyfin-RPC in action</summary>
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

</details>

## Setup
#### Prerequisites
- A Jellyfin server
	- Account on said server
	- Jellyfin API key on said server
- Discord
- Imgur API key (image support without port forwarding)

A fully filled out config would look something like this

```
{
    "jellyfin": {
        "url": "https://example.com",
        "api_key": "sadasodsapasdskd",
        "username": ["my_first_user", "my_second_user"],
        "music": {
            "display": ["year", "album", "genres"],
            "separator": "-"
        },
        "blacklist": {
            "media_types": ["music", "movie", "episode", "livetv"],
            "libraries": ["Anime", "Anime Movies"]
        }
    },
    "discord": {
        "application_id": "1053747938519679018",
        "buttons": [
            {
                "name": "dynamic",
                "url": "dynamic"
            },
            {
                "name": "dynamic",
                "url": "dynamic"
            }
        ]
    },
    "imgur": {
        "client_id": "asdjdjdg394209fdjs093"
    },
    "images": {
        "enable_images": true,
        "imgur_images": true
    }
}
```


but all of that isn't needed to make the code work, for that you'd only need this.

```
{
	"jellyfin": {
        "url": "https://example.com",
        "api_key": "sadasodsapasdskd",
        "username": "your_username_here",
    }
    "images": {
        "enable_images": false,
        "imgur_images": false
    }
}
```

Not that much right? That's because there are defaults for most of these options, with this barebones config you wouldn't get any images but apart from that it's fully functional.

Now then, lets continue on with the setup

### Jellyfin server URL
The first thing you need is the url to your server, it will look something like one of these
- `http://192.168.1.2:8096`
- `http://[2001:4647:aa09:0:b62e:99ff:fe15:984d]:8096`
- `https://jf.radical.fun`
###### NOTE: The http/https part is important, without it the script will crash

![URL shown in browser with red arrow pointed at it](https://github.com/Radiicall/jellyfin-rpc/assets/66682497/d3c11318-1b24-4f50-b36b-119d60ea59ed)

### Jellyfin API key
The next thing you'll need is an API key, you can get one at
http(s)://your_jellyfin_url/web/#!/apikeys.html

1. Click the plus here

![API Key page with arrow pointed to plus button](https://github.com/Radiicall/jellyfin-rpc/assets/66682497/6dc2492f-4c95-487a-96e2-dd11ce89f520)

2. Choose a name

![Picture of API Key creation UI](https://github.com/Radiicall/jellyfin-rpc/assets/66682497/fed20047-d285-4d6a-912e-abcfc2a1991c)

3. Copy the key

![Arrow pointed at newly created API key](https://github.com/Radiicall/jellyfin-rpc/assets/66682497/da494f07-7414-4683-8a2b-00cc02cb2930)

### Jellyfin username
You also need your username, this is the one you use to log in
![Username entered into login screen of Jellyfin](https://github.com/Radiicall/jellyfin-rpc/assets/66682497/8cefb179-4ed4-418c-9ea0-b60702aede17)

### OPTIONAL

### Discord Application ID
You can make a discord application by going <a href="https://discord.com/developers/applications">here</a>.

1. Click "New Application"

![Arrow pointing to "New Application" button](https://github.com/Radiicall/jellyfin-rpc/assets/66682497/b3cbce7e-0eca-4a8a-98f2-a0f9f3e25c8d)

2. then click "Create"

![Creation screen](https://github.com/Radiicall/jellyfin-rpc/assets/66682497/f784fb96-0ff6-410d-a041-76c614a1ce08)

3. then click "Copy" on the Application ID

![Red box around newly created application id](https://github.com/Radiicall/jellyfin-rpc/assets/66682497/2d1733eb-738b-4494-b3e7-35d991b49c2e)

### Imgur API
For the imgur api to work you have to do this in the config
```
"images": {
    "enable_images": true,
    "imgur_images": true
}
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
