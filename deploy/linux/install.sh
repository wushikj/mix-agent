#!/bin/bash
# Print script introduction information
echo '------------------------------------------------------------------------------------------'
echo 'Copyright @2020 for Wushi (Fujian) Technology Co., Ltd'
echo 'This shell script will install single mix agent'
echo "Current working directory is: $(pwd)"
echo '------------------------------------------------------------------------------------------'

if [ "$1" = "" ]
then
 echo "Please input agent name, eg: mix_agent_machine"
 return
fi

echo "Install $1"
eval "nohup ./${1} >/dev/null 2>&1 &"
echo 'Install completed'

# view install result
ps aux | grep -v grep | grep -v 'install.sh' | grep mix_agent