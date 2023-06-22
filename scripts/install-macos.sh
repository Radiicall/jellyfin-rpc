#!/bin/zsh
# A script to autoamtically install jellyfin-rpc on mac
clear

# Get the user's server information
vared -p "Jellyfin Server URL (include http/https): " -c jellyfinurl
vared -p "Jellyfin API Key (you can find this at ${jellyfinurl}/web/#!/apikeys.html): " -c jellyfinkey
vared -p "Jellyfin Username: " -c jellyfinuser
echo ""

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
  typeBlocklistString=''
else
  typeBlocklistString='['
  for i in ${responses[@]}; do
    typeBlocklistString+="\"$i\", "
  done
  typeBlocklistString=${typeBlocklistString%?}
  typeBlocklistString=${typeBlocklistString%?}
  typeBlocklistString+=']'
fi
echo ""

# Prompt for libraries to block
echo "Separated by only commas, type the names of any libraries to be excluded from Jellyfin RPC. Press return when finished."
echo "For example, typing 'Anime,Anime Movies' will filter out media from libraries named 'Anime' and 'Anime Movies'."
vared -p "Leave blank to enable all libraries. This will not disable filtering by media type. " -c libBlocklist

# Build the blocklist string
libBlocklistArray=(${(@s:,:)libBlocklist})
if [[ ${#libBlocklistArray} -eq 0 ]]; then
  libBlocklistString=''
else
  libBlocklistString='['
  for i in ${libBlocklistArray[@]}; do
    libBlocklistString+="\"$i\", "
  done
  libBlocklistString=${libBlocklistString%?}
  libBlocklistString=${libBlocklistString%?}
  libBlocklistString+=']'
fi
echo ""

# Get discord application ID, or use default if left blank
vared -p "Discord Application ID (leave blank if you're unsure): " -c discordAppId
if [ -z $discordAppId ]; then
  discordAppId="1053747938519679018"
fi

# Enable or disable image uploading
configImagesEnabled="false"
configImgurEnabled="false"
vared -p "Display media images in discord? (y/n) " -c artworkEnabled
if [[ $artworkEnabled == [Yy]* ]]; then
  configImagesEnabled="true"

  # Get Imgur client ID & enable Imgur uploading
  echo "If the server is not port-forwarded, you must enable uploading artwork to Imgur or images will not work."
  vared -p "Upload media images to Imgur to be displayed in discord? Selecting \"n\" suggests that your server is port-forwarded! (y/n) " -c imgurEnabled
  if [[ $imgurEnabled == [Yy]* ]]; then
    configImgurEnabled="true"
    echo "To get an Imgur client ID, go to https://api.imgur.com/oauth2/addclient"
    echo "Name can be anything, authorization type must be 'OAuth 2 authorization without a callback URL'"
    echo "Press submit to get your client ID"
    vared -p "Imgur client ID (): " -c imgurId
  fi
fi

# Put together the config file

configFileContents=""
configFileContents+="$(cat <<EOF
{
    "jellyfin": {
        "url": "${jellyfinurl}",
        "api_key": "${jellyfinkey}",
        "username": "${jellyfinuser}"
EOF
)"
if [[ ${#responses} -ne 0 ]]; then # only add this line if there are items in the blocklist
  configFileContents+="$(cat <<EOF
,
        "type_blacklist": ${typeBlocklistString}
EOF
)"
fi
if [[ ${#libBlocklistArray} -ne 0 ]]; then # only add this line if there are items in the library blocklist
  configFileContents+="$(cat <<EOF
,
        "library_blacklist": ${libBlocklistString}
EOF
)"
fi
configFileContents+="$(cat <<EOF

    },
    "discord": {
        "application_id": "${discordAppId}"
    },
    "imgur": {
        "client_id": "${imgurId}"
    },
    "images": {
        "enable_images": ${configImagesEnabled},
        "imgur_images": ${configImgurEnabled}
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

# Save config to file
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

    if pgrep -xq -- "jellyfin-rpc"; then
        killall jellyfin-rpc # Kill jellyfin-rpc if it is running
    fi
    if launchctl list | grep Jellyfin-RPC  &> /dev/null; then
        launchctl remove Jellyfin-RPC # Unload Jellyfin-RPC LaunchAgent if it is loaded
    fi

    # Create LaunchAgent file
    cat > ~/Library/LaunchAgents/jellyfinrpc.local.plist<< EOF
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
    chmod 644 ~/Library/LaunchAgents/jellyfinrpc.local.plist # Give LaunchAgent proper permissions
    launchctl load ~/Library/LaunchAgents/jellyfinrpc.local.plist # Load LaunchAgent
  fi
  echo "Jellyfin RPC is now set up to start at login."
  echo "If needed, you can run Jellyfin RPC at any time by running 'jellyfin-rpc' in a terminal."
fi

exit