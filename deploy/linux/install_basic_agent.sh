#!/bin/bash
# Print script introduction information
echo '------------------------------------------------------------------------------------------'
echo 'Copyright @2020 for Wushi (Fujian) Technology Co., Ltd'
echo 'This shell script will install basic mix agent'
echo 'Current working directory is:' $(pwd)
echo '------------------------------------------------------------------------------------------'

# install basic agent: cpu/memory/disk/machine
echo 'Install mix_agent_machine'
nohup ./mix_agent_machine >/dev/null 2>&1 &

sleep 5

echo 'Install mix_agent_cpu'
nohup ./mix_agent_cpu >/dev/null 2>&1 &

echo 'Install mix_agent_memory'
nohup ./mix_agent_memory >/dev/null 2>&1 &

echo 'Install mix_agent_disk'
nohup ./mix_agent_disk >/dev/null 2>&1 &
echo 'Install completed'

# view install result
ps aux | grep -v grep | grep mix_agent