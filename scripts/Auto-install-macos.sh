#!/bin/zsh
# A script to autoamtically install jellyfin-rpc on mac
clear

# Get the user's server information
vared -p "Jellyfin Server URL (include http/https): " -c jellyfinurl
vared -p "Jellyfin API Key (you can find this at ${jellyfinurl}/web/#!/apikeys.html): " -c jellyfinkey
vared -p "Jellyfin Username: " -c jellyfinuser

# Prompt the user for what libraries should be included/blocked
responses=()

vared -p "Include Movies in Discord Rich Presence? (y/n) " -c moviesEnabled
if [[ $moviesEnabled == [Nn]* ]]; then
  responses+=( "movie" )
fi

vared -p "Include TV in Discord Rich Presence? (y/n) " -c tvEnabled
if [[ $tvEnabled == [Nn]* ]]; then
  responses+=( "episode" )
fi

vared -p "Include Music in Discord Rich Presence? (y/n) " -c musicEnabled
if [[ $musicEnabled == [Nn]* ]]; then
  responses+=( "music" )
fi

vared -p "Include Live TV in Discord Rich Presence? (y/n) " -c livetvEnabled
if [[ $livetvEnabled == [Nn]* ]]; then
  responses+=( "livetv" )
fi

# Build the blocklist string
if [[ ${#responses} -eq 0 ]]; then
  blocklistString='["movie", "episode", "music", "livetv"]'
else
  blocklistString='['
  for i in ${responses[@]}; do
    blocklistString+="\"$i\", "
  done
  blocklistString=${blocklistString%?}
  blocklistString=${blocklistString%?}
  blocklistString+=']'
fi

#get discord application ID, or use default if left blank
vared -p "Discord Application ID (leave blank if you're unsure): " -c discordAppId
if [ -z $discordAppId ]; then
  discordAppId="1053747938519679018"
fi

# Get Imgur client ID & enable/disable image uploading
configImgurEnabled="false"
vared -p "Upload album artwork to Imgur to be displayed in discord? (y/n) " -c artworkEnabled
if [[ $artworkEnabled == [Yy]* ]]; then
  configImgurEnabled="true"
  echo "To get an Imgur client ID, go to https://api.imgur.com/oauth2/addclient"
  echo "Name can be anything, authorization type must be 'OAuth 2 authorization without a callback URL'"
  echo "Press submit to get your client ID"
  echo "NOTE: Port formwarding must be enabled on your server to send images"
  vared -p "Imgur client ID (): " -c imgurId
fi

#put together the config file

configFileContents=""
configFileContents+="$(cat <<EOF
{
    "Jellyfin": {
        "URL": "${jellyfinurl}",
        "API_KEY": "${jellyfinkey}",
        "USERNAME": "${jellyfinuser}"
EOF
)"
if [[ ${#responses} -ne 0 ]]; then # only add this line if there are items in the blocklist
  configFileContents+="$(cat <<EOF
,
        "BLOCKLIST": ${blocklistString}
EOF
)"
fi
configFileContents+="$(cat <<EOF

    },
    "Discord": {
        "APPLICATION_ID": "${discordAppId}"
    },
    "Imgur": {
        "CLIENT_ID": "${imgurId}"
    },
    "Images": {
        "ENABLE_IMAGES": ${configImgurEnabled},
        "IMGUR_IMAGES": ${configImgurEnabled}
    }
}
EOF
)"

# Preview config file and have user confirm before saving it
echo "Config file preview:"
echo $configFileContents
vared -p "Save it and proceed? (y/n) " -c proceed
if [[ $proceed == [Nn]* ]]; then
  exit
fi

#Save config to file
if [ ! -d ~/.config/jellyfin-rpc ]; then
  mkdir ~/.config/jellyfin-rpc
fi
echo $configFileContents > ~/.config/jellyfin-rpc/main.json

# Prompt user to install Jellyfin RPC
vared -p "Download latest release? (y/n) " -c downloadlatest
if [[ $downloadlatest == [Yy]* ]]; then
  # download file to binary directory and give execution permissions
  curl -o /usr/local/bin/jellyfin-rpc -L https://github.com/Radiicall/jellyfin-rpc/releases/latest/download/jellyfin-rpc-x86_64-darwin
  chmod +x /usr/local/bin/jellyfin-rpc

  # Prompt user to enabled Jellyfin RPC running at login
  vared -p "Set Jellyfin-RPC to run at login? (y/n) " -c runAtLogin
  if [[ $runAtLogin == [Yy]* ]]; then
    # Create launch agent file, give it proper permissions, then load it.
    cat >> ~/Library/LaunchAgents/jellyfinrpc.local.plist<< EOF
<?xml version="1.0" encoding="UTF-8"?>
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
</plist>
EOF
    chmod 644 ~/Library/LaunchAgents/jellyfinrpc.local.plist
    launchctl unload ~/Library/LaunchAgents/jellyfinrpc.local.plist #It seems that the OS tries to load the launch agent on creation, so it needs unloaded first.
    launchctl load ~/Library/LaunchAgents/jellyfinrpc.local.plist
  fi
  echo "If needed, you can run Jellyfin RPC at any time by running 'jellyfin-rpc' in a terminal"
fi

exit