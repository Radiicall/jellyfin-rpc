#!/bin/zsh
# A script to autoamtically uninstall jellyfin-rpc on mac
if pgrep -xq -- "jellyfin-rpc"; then
    killall jellyfin-rpc # Kill jellyfin-rpc if it is running
fi
if launchctl list | grep Jellyfin-RPC; then
    launchctl remove Jellyfin-RPC # Unload Jellyfin-RPC launchagent if it is loaded
fi
rm ~/Library/LaunchAgents/jellyfinrpc.local.plist #remove launch agent
rm -rf ~/.config/jellyfin-rpc #remove config file
rm /usr/local/bin/jellyfin-rpc #remove binary
echo "Uninstall complete!"
