# Jellyfin-RPC
Program used to display what you're currently watching on discord.

Jellyfin-RPC uses the API to check what you're currently watching, this means that the program can be ran from a server or your computer. The only requirement is that discord is open and logged in.


Example Movie:

![image](https://user-images.githubusercontent.com/66682497/209231361-411296d1-031c-4a87-bcdf-87efde6f3ada.png)

Example Series:

![image](https://user-images.githubusercontent.com/66682497/209229842-350d9fba-cf29-461e-9a0c-3bc47ec24389.png)

Terminal Output:

![image](https://user-images.githubusercontent.com/66682497/208316064-0d66b0cc-2529-4947-8ea9-5b0f48df16e4.png)

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
```

### Discord Application ID
This step is optional as I have included my own.

You can make a discord application by going <a href="https://discord.com/developers/applications">here</a>.

### Jellyfin URL
This will be the URL to your jellyfin instance, 

if you want to know more about jellyfin you can check it out <a href="https://jellyfin.org/">here</a>.

### Jellyfin API Key
This is the API key used for checking what you're currently watching on Jellyfin.

You can get one by going to example.com/web/#!/apikeys.html

Replace "example.com" with your instance URL.

### Jellyfin Username
This is the username you use to log into Jellyfin.

The username is needed because if you have multiple accounts (friends, family) then the program will just grab the first person it sees in the list.

## Building
You need rust installed, you can get rustup from <a href="https://rustup.rs/">here</a>

If you already have rustup installed then make sure its the latest 2021 version, you can run `rustup update` to update to the newest version.

You also need openssl libs on linux (Don't remember exactly which one, running the jellyfin-rpc exec will tell you what you're missing)

Please make an issue with the missing libs in this repo so i can put them here, also say what distro you're running so i can test it.

After doing all of this you should be able to just run `cargo build` to get a binary.
In order to get an optimized binary just add `--release` to the end of cargo build.

Your binary will be located in `target/debug/jellyfin-rpc` or `target/release/jellyfin-rpc`
