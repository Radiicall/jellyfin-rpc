# Jellyfin-RPC
Program used to display what you're currently watching on discord.

Jellyfin-RPC uses the API to check what you're currently watching, this means that the program can be ran from a server or your computer. The only requirement is that discord is open and logged in.


Example Movie:

![image](https://user-images.githubusercontent.com/66682497/208316089-d9e19ae1-6587-4774-a5ef-202bc64d6a04.png)

Example Series:

![image](https://user-images.githubusercontent.com/66682497/208309168-6c4870c4-4149-4c3d-ae70-9b0855652663.png)

Terminal Output:

![image](https://user-images.githubusercontent.com/66682497/208316064-0d66b0cc-2529-4947-8ea9-5b0f48df16e4.png)


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
