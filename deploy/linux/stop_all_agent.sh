#!/bin/bash
# Print script introduction information
echo '------------------------------------------------------------------------------------------'
echo 'Copyright @2020 for Wushi (Fujian) Technology Co., Ltd'
echo 'This shell script will (stop)kill all mix agent'
echo "Current working directory is: $(pwd)"
echo '------------------------------------------------------------------------------------------'

# kill all mix agent
ps aux | grep mix_agent | grep -v grep | awk '{print $2} ' | xargs kill -9
echo 'All mix agent is killed'

# view install result
ps aux | grep -v grep | grep mix_agent