#!/bin/zsh
# A script to autoamtically uninstall jellyfin-rpc on mac
killall jellyfin-rpc #stop any running processes
rm ~/Library/LaunchAgents/jellyfinrpc.local.plist #remove launch agent
rm -rf ~/.config/jellyfin-rpc #remove config file
rm /usr/local/bin/jellyfin-rpc #remove binary