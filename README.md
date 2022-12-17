# Jellyfin-RPC
Program used to display what you're currently watching on discord.

Jellyfin-RPC uses the API to check what you're currently watching, this means that the program can be ran from a server or your computer. The only requirement is that discord is open and logged in.


Example Movie:

![image](https://user-images.githubusercontent.com/66682497/208263440-d8842ecb-0502-42fe-85bd-15e16eb86b48.png)

Example Series:

![image](https://user-images.githubusercontent.com/66682497/208263467-851e31de-1e7d-4000-aec3-ce1f440bd1e7.png)

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

Replace "example.com" with the actual URL

### Jellyfin Username
This is the username you use to log into Jellyfin.

The username is needed because if you have multiple accounts (friends, family) then the program will just grab the first person it sees in the list.
