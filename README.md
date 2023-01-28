# Jellyfin-RPC

[Frequently Asked Questions](FAQ.md)

Program used to display what you're currently watching on discord.

Jellyfin-RPC uses the API to check what you're currently watching, this means that the program can be ran from a server or your computer. The only requirement is that discord is open and logged in.


Example Movie:

![image](https://user-images.githubusercontent.com/66682497/213467832-5eb6b0a0-1b83-47db-bf00-48c0e739aec4.png)

Example Series:

![image](https://user-images.githubusercontent.com/66682497/213467669-8375841d-b846-4afe-8bd3-0b09f4c7f2ad.png)

Terminal Output:

<img width="474" alt="image" src="https://user-images.githubusercontent.com/66682497/214524256-7347df00-9247-4140-814d-569055ce39f8.png">

This program is very memory/cpu efficient using ~13mb of ram and ~0.1% of the cpu while sending info to discord.

It's even better than previous versions of the code, before it would always sit on 0.1%-0.2% but now it sits on 0.0%-0.1%,
the ram usage has increased by 1 megabyte however.

```
CPU: Ryzen 5 3600XT@4.4Ghz
Mem: 32GB
```

![image](https://user-images.githubusercontent.com/66682497/211466607-6482a37c-3cf8-434c-a282-85c53e84697e.png)

## Setup
Make a .env file with the following items
```
DISCORD_APPLICATION_ID=1053747938519679018
JELLYFIN_URL=your_url_here
JELLYFIN_API_KEY=your_api_key_here
JELLYFIN_USERNAME=your_username_here
ENABLE_IMAGES=true
```

### Discord Application ID
This step is optional as I have included my own.

If this env var is empty in the .env file then it will also use the default one

You can make a discord application by going <a href="https://discord.com/developers/applications">here</a>.

### Jellyfin URL
This will be the URL to your jellyfin instance, remember to include http/https in the url.

If you want to know more about jellyfin you can check it out <a href="https://jellyfin.org/">here</a>.

### Jellyfin API Key
This is the API key used for checking what you're currently watching on Jellyfin.

You can get one by going to \<YOUR INSTANCE URL HERE>/web/#!/apikeys.html

### Jellyfin Username
This is the username you use to log into Jellyfin.

The username is needed because if you have multiple accounts (friends, family) then the program will just grab the first person it sees in the list.

### Systemd

For systemd I have included <a href="https://raw.githubusercontent.com/Radiicall/jellyfin-rpc/main/jellyfin-rpc.service">this file</a>, you can download it directly by pressing ctrl+s on the page.

In the service file you have to change the `ExecStart=` line. You can launch the script without -c if you put the `.env` file in the same directory as the executable.

The service is supposed to run in user mode, put the service file into this directory `$HOME/.config/systemd/user/` and use `systemctl --user enable --now jellyfin-rpc.service` to start and enable it.

## Building
You need rust installed, you can get rustup from <a href="https://rustup.rs/">here</a>

If you already have rustup installed then make sure its the latest 2021 version, you can run `rustup update` to update to the newest version.

You also need openssl libs on linux (Don't remember exactly which one, running the jellyfin-rpc exec will tell you what you're missing)

Please make an issue with the missing libs in this repo so I can put them here, also say what distro you're running so I can test it.

After doing all of this you should be able to just run `cargo build` to get a binary.
In order to get an optimized binary just add `--release` to the end of cargo build.

Your binary will be located in `target/debug/jellyfin-rpc` or `target/release/jellyfin-rpc`
