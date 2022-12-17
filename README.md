# Jellyfin-RPC
Program used to display what you're currently watching on discord

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

You can make a discord application by going <a href="">here</a>.

### Jellyfin URL
This will be the URL to your jellyfin instance, 

if you want to know more about jellyfin you can check it out <a href="">here</a>.

### Jellyfin API Key
This is the API key used for checking what you're currently watching on Jellyfin.

You can get one by going to <your instance url>/web/#!/apikeys.html

### Jellyfin Username
This is the username you use to log into Jellyfin.

The username is needed because if you have multiple accounts (friends, family) then the program will just grab the first person it sees in the list.
